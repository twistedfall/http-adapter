pub use async_trait;
pub use http::{self, Request};

/// Adapter to allow different HTTP clients to be used to issue an HTTP request.
/// To properly implement this trait, use [async_trait](https://crates.io/crates/async-trait).
#[async_trait::async_trait(? Send)]
pub trait HttpClientAdapter {
	/// Error type used by the underlying HTTP library
	type Error;

	/// Fetch the specified URL using the specified request method
	///
	/// Returns the text contents of the resource located at the indicated URL
	async fn execute(&self, request: Request<Vec<u8>>) -> Result<Vec<u8>, Self::Error>;
}
