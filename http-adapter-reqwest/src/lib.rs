use http_adapter::async_trait::async_trait;
use http_adapter::http::{Request, Response};
use http_adapter::{http, HttpClientAdapter};
use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

#[derive(Clone, Debug)]
pub struct ReqwestAdapter {
	client: reqwest::Client,
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
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			Error::Http(e) => Display::fmt(e, f),
			Error::Reqwest(e) => Display::fmt(e, f),
		}
	}
}

impl StdError for Error {}

#[inline]
async fn to_response(res: reqwest::Response) -> Result<Response<Vec<u8>>, Error> {
	let mut response = Response::builder().status(res.status()).version(res.version());
	if let Some(headers) = response.headers_mut() {
		headers.clone_from(res.headers());
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
			.execute(request.try_into().map_err(Error::Reqwest)?)
			.await
			.map_err(Error::Reqwest)?;

		to_response(res).await
	}
}
