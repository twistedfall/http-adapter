use http_adapter::http::StatusCode;
use http_adapter::{HttpClientAdapter, Request};
use http_adapter_ureq::UreqAdapter;

#[tokio::test]
async fn test_ureq() {
	let client = UreqAdapter::default();
	let resp = client
		.execute(Request::get("https://www.example.com").body(b"".to_vec()).unwrap())
		.await
		.unwrap();
	assert_eq!(StatusCode::OK, resp.status());
	assert!(!resp.body().is_empty());
}
