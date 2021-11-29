use http_adapter::{async_trait::async_trait, HttpClientAdapter, http::Request};

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

#[async_trait(?Send)]
impl HttpClientAdapter for ReqwestAdapter {
	type Error = reqwest::Error;

	async fn execute(&self, request: Request<Vec<u8>>) -> Result<Vec<u8>, Self::Error> {
		let res = self.client
			.execute(request.try_into()?)
			.await?
			.error_for_status()?;
		Ok(res.bytes().await?.to_vec())
	}
}
