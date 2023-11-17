//! # HTTP adapter implementation for [`reqwest`](https://crates.io/crates/reqwest)
//!
//! For more details refer to [`http-adapter`](https://crates.io/crates/http-adapter)

use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::str::FromStr;

pub use reqwest;

use http_adapter::async_trait::async_trait;
use http_adapter::http::{HeaderName, HeaderValue, Request, Response, StatusCode, Version};
use http_adapter::{http, HttpClientAdapter};

#[derive(Clone, Debug)]
pub struct ReqwestAdapter {
	client: reqwest::Client,
}

impl ReqwestAdapter {
	pub fn new(client: reqwest::Client) -> Self {
		Self { client }
	}
}

impl Default for ReqwestAdapter {
	#[inline]
	fn default() -> Self {
		Self {
			client: reqwest::Client::new(),
		}
	}
}

#[derive(Debug)]
pub enum Error {
	Http(http::Error),
	Reqwest(reqwest::Error),
	InvalidMethod(String),
	InvalidHeaderName(String),
	InvalidHeaderValue(Vec<u8>),
	InvalidHttpVersion(String),
	InvalidStatusCode(u16),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			Error::Http(e) => Display::fmt(e, f),
			Error::Reqwest(e) => Display::fmt(e, f),
			Error::InvalidMethod(method) => {
				write!(f, "Invalid method: {method}")
			}
			Error::InvalidHeaderName(name) => {
				write!(f, "Invalid header name: {name}")
			}
			Error::InvalidHeaderValue(value) => {
				write!(f, "Invalid header value: {value:?}")
			}
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
fn from_request<B: Into<reqwest::Body>>(client: &reqwest::Client, request: Request<B>) -> Result<reqwest::Request, Error> {
	let method_str = request.method().as_str();
	let version = match request.version() {
		Version::HTTP_09 => reqwest::Version::HTTP_09,
		Version::HTTP_10 => reqwest::Version::HTTP_10,
		Version::HTTP_11 => reqwest::Version::HTTP_11,
		Version::HTTP_2 => reqwest::Version::HTTP_2,
		Version::HTTP_3 => reqwest::Version::HTTP_3,
		_ => return Err(Error::InvalidHttpVersion(format!("{:?}", request.version()))),
	};
	let mut out = client
		.request(
			reqwest::Method::from_str(method_str).map_err(|_| Error::InvalidMethod(method_str.to_string()))?,
			request.uri().to_string(),
		)
		.version(version);
	for (name, value) in request.headers() {
		out = out.header(
			name.as_str(),
			value
				.to_str()
				.map_err(|_| Error::InvalidHeaderValue(value.as_bytes().to_vec()))?,
		);
	}
	out.body(request.into_body()).build().map_err(Error::Reqwest)
}

#[inline]
async fn to_response(res: reqwest::Response) -> Result<Response<Vec<u8>>, Error> {
	let version = match res.version() {
		reqwest::Version::HTTP_09 => Version::HTTP_09,
		reqwest::Version::HTTP_10 => Version::HTTP_10,
		reqwest::Version::HTTP_11 => Version::HTTP_11,
		reqwest::Version::HTTP_2 => Version::HTTP_2,
		reqwest::Version::HTTP_3 => Version::HTTP_3,
		_ => return Err(Error::InvalidHttpVersion(format!("{:?}", res.version()))),
	};
	let status_code = res.status().as_u16();
	let status = StatusCode::from_u16(status_code).map_err(|_| Error::InvalidStatusCode(status_code))?;
	let mut response = Response::builder().status(status).version(version);
	if let Some(headers) = response.headers_mut() {
		for (name, value) in res.headers() {
			let name_str = name.as_str();
			let value_bytes = value.as_bytes();
			headers.insert(
				HeaderName::from_str(name_str).map_err(|_| Error::InvalidHeaderName(name_str.to_string()))?,
				HeaderValue::from_bytes(value_bytes).map_err(|_| Error::InvalidHeaderValue(value_bytes.to_vec()))?,
			);
		}
	}
	response
		.body(res.bytes().await.map_err(Error::Reqwest)?.to_vec())
		.map_err(Error::Http)
}

#[async_trait(?Send)]
impl HttpClientAdapter for ReqwestAdapter {
	type Error = Error;

	async fn execute(&self, request: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, Self::Error> {
		let res = self
			.client
			.execute(from_request(&self.client, request)?)
			.await
			.map_err(Error::Reqwest)?;
		to_response(res).await
	}
}
