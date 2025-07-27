//! # HTTP adapter implementation for [`reqwest`](https://crates.io/crates/reqwest)
//!
//! For more details refer to [`http-adapter`](https://crates.io/crates/http-adapter)

use std::fmt::Debug;

use http_body_util::BodyExt;
pub use reqwest;

use http_adapter::async_trait::async_trait;
use http_adapter::http::{Request, Response};
use http_adapter::HttpClientAdapter;

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

#[inline]
fn from_request<B: Into<reqwest::Body>>(request: Request<B>) -> Result<reqwest::Request, reqwest::Error> {
	request.try_into()
}

#[inline]
async fn to_response(res: reqwest::Response) -> Result<Response<Vec<u8>>, reqwest::Error> {
	let mut res = Response::from(res);
	let body_bytes = res.body_mut().collect().await?.to_bytes().to_vec();
	Ok(res.map(move |_| body_bytes))
}

#[async_trait]
impl HttpClientAdapter for ReqwestAdapter {
	type Error = reqwest::Error;

	async fn execute(&self, request: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, Self::Error> {
		let res = self.client.execute(from_request(request)?).await?;
		to_response(res).await
	}
}
