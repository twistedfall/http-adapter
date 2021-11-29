use surf::{
	http::Method as SurfMethod,
	StatusCode,
};

use http_adapter::{
	async_trait::async_trait,
	HttpClientAdapter,
	http::{Method, Request},
};

#[derive(Clone, Debug)]
pub struct SurfAdapter {
	client: surf::Client,
}

impl Default for SurfAdapter {
	#[inline]
	fn default() -> Self {
		Self {
			client: surf::Client::new(),
		}
	}
}

#[inline]
fn from_request(request: Request<Vec<u8>>) -> Result<surf::Request, surf::Error> {
	let (head, body) = request.into_parts();
	let method = match head.method {
		Method::GET => SurfMethod::Get,
		Method::POST => SurfMethod::Post,
		Method::PUT => SurfMethod::Put,
		Method::PATCH => SurfMethod::Patch,
		Method::DELETE => SurfMethod::Delete,
		Method::HEAD => SurfMethod::Head,
		Method::CONNECT => SurfMethod::Connect,
		Method::OPTIONS => SurfMethod::Options,
		Method::TRACE => SurfMethod::Trace,
		_ => return Err(surf::Error::from_str(
			StatusCode::MethodNotAllowed,
			format!("Method: {} is not supported by the crate: `{}`", head.method, env!("CARGO_CRATE_NAME")),
		)),
	};
	let mut out = surf::Request::new(method, surf::Url::parse(&head.uri.to_string())?);
	for (name, value) in &head.headers {
		out.append_header(name.as_str(), value.to_str()?);
	}
	out.body_bytes(body);
	Ok(out)
}

#[async_trait(?Send)]
impl HttpClientAdapter for SurfAdapter {
	type Error = surf::Error;

	async fn execute(&self, request: Request<Vec<u8>>) -> Result<Vec<u8>, Self::Error> {
		let req = from_request(request)?;
		self.client.recv_bytes(req).await
	}
}
