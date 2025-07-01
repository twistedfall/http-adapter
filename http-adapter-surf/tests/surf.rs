use http_adapter::http::StatusCode;
use http_adapter::{HttpClientAdapter, Request};
use http_adapter_surf::SurfAdapter;

#[tokio::test]
async fn test_surf() {
	let client = SurfAdapter::default();
	let resp = client
		.execute(Request::get("https://www.example.com").body(b"".to_vec()).unwrap())
		.await
		.unwrap();
	assert_eq!(StatusCode::OK, resp.status());
	assert!(!resp.body().is_empty());
}
