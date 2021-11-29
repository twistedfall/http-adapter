use std::io::Read;

use http_adapter::{
	async_trait::async_trait,
	HttpClientAdapter,
	http::Request,
};

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

#[inline]
fn from_request<B>(request: &Request<B>) -> Result<ureq::Request, ureq::Error> {
	let mut out = ureq::request(request.method().as_str(), &request.uri().to_string());
	for (name, value) in request.headers() {
		if let Ok(value) = value.to_str() {
			out = out.set(name.as_str(), value);
		}
	}
	Ok(out)
}

#[async_trait(?Send)]
impl HttpClientAdapter for UreqAdapter {
	type Error = ureq::Error;

	async fn execute(&self, request: Request<Vec<u8>>) -> Result<Vec<u8>, Self::Error> {
		let res = from_request(&request)?
			.send_bytes(request.body())?;
		let mut out = vec![];
		res.into_reader().read_to_end(&mut out)?;
		Ok(out)
	}
}
