use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::io::Read;

use http_adapter::async_trait::async_trait;
use http_adapter::http::{StatusCode, Version};
use http_adapter::{http, HttpClientAdapter};
use http_adapter::{Request, Response};

#[derive(Clone, Debug)]
pub struct UreqAdapter {
	client: ureq::Agent,
}

impl Default for UreqAdapter {
	#[inline]
	fn default() -> Self {
		Self {
			client: ureq::Agent::new(),
		}
	}
}

#[derive(Debug)]
pub enum Error {
	Http(http::Error),
	Ureq(ureq::Error),
	InvalidHttpVersion(String),
	InvalidStatusCode(u16),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			Error::Http(e) => Display::fmt(e, f),
			Error::Ureq(e) => Display::fmt(e, f),
			Error::InvalidHttpVersion(version) => {
				write!(f, "Invalid HTTP version: {version}")
			}
			Error::InvalidStatusCode(code) => {
				write!(f, "Invalid status code: {code}")
			}
		}
	}
}

impl StdError for Error {}

#[inline]
fn from_request<B>(client: &ureq::Agent, request: &Request<B>) -> Result<ureq::Request, ureq::Error> {
	let mut out = client.request(request.method().as_str(), &request.uri().to_string());
	for (name, value) in request.headers() {
		if let Ok(value) = value.to_str() {
			out = out.set(name.as_str(), value);
		}
	}
	Ok(out)
}

#[inline]
fn to_response(res: ureq::Response) -> Result<Response<Vec<u8>>, Error> {
	let version = match res.http_version() {
		"HTTP/0.9" => Version::HTTP_09,
		"HTTP/1.0" => Version::HTTP_10,
		"HTTP/1.1" => Version::HTTP_11,
		"HTTP/2.0" => Version::HTTP_2,
		"HTTP/3.0" => Version::HTTP_3,
		_ => return Err(Error::InvalidHttpVersion(res.http_version().to_string())),
	};

	let status = StatusCode::from_u16(res.status()).map_err(|_| Error::InvalidStatusCode(res.status()))?;

	let mut response = Response::builder().status(status).version(version);

	for header_name in res.headers_names() {
		if let Some(header_value) = res.header(&header_name) {
			response = response.header(header_name, header_value);
		}
	}
	let mut body = vec![];
	res.into_reader().read_to_end(&mut body).map_err(|e| Error::Ureq(e.into()))?;
	response.body(body).map_err(Error::Http)
}

#[async_trait(?Send)]
impl HttpClientAdapter for UreqAdapter {
	type Error = Error;

	async fn execute(&self, request: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, Self::Error> {
		let res = from_request(&self.client, &request)
			.map_err(Error::Ureq)?
			.send_bytes(request.body())
			.map_err(Error::Ureq)?;
		to_response(res)
	}
}
