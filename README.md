# http-adapter

## Documentation

See [full documentation](https://docs.rs/http-adapter)

## Usage

Add this to your Cargo.toml:
```
[dependencies]
http-adapter = "0.1.0"
```

![Maintenance](https://img.shields.io/badge/maintenance-passively--maintained-yellowgreen.svg)
[![Build Status](https://github.com/twistedfall/http-adapter/actions/workflows/http-adapter.yml/badge.svg)](https://github.com/twistedfall/http-adapter/actions/workflows/http-adapter.yml)
[![Documentation](https://docs.rs/http-adapter/badge.svg)](https://docs.rs/http-adapter)

## Adapter for HTTP client agnostic requests

This crate allows the async libraries to be HTTP client agnostic and not force a particular choice of an async runtime because
of it. It is mostly useful when you develop a crate that provides access to an HTTP API.

If you need to do somewhat simple HTTP requests (multiple HTTP methods, header/cookie management is included), want to be HTTP
client agnostic and not lock the downstream users to a particular async runtime (think `tokio`, `async-std`, `smol`) then you
need to make sure that your API client accepts an instance of the generic type implementing [`HttpClientAdapter`] and use it
to make HTTP requests. Users of your crate will then use a particular adapter based on the actual HTTP client (e.g. `request`,
`ureq`, `surf`, etc.) and supply it when creating your API client.

## Usage

The [`HttpClientAdapter`] trait exposes a single async function [`HttpClientAdapter::execute()`]. It takes an instance of
[`http::Request`] that encodes the necessary request parameters like HTTP method and URL and executes it. Then it returns an
instance of [`http::Response`] with the server response. The request and response types come from the
[`http`](https://crates.io/crates/http) crate. The body of the request and response are expected to be `Vec<u8>`.

## Adapter implementation

To create a new implementation of [`HttpClientAdapter`] for an HTTP client library please refer to the following crates:
  * [`http-adapter-reqwest`][1] - async wrapper, simple case because `reqwest` is using `http` types internally
  * [`http-adapter-surf`][2] - async wrapper, more complicated case because of the need to convert types
  * ['http-adapter-ureq`][3] - sync wrapper, complex case because of the need to wrap a sync client in an runtime-agnostic fashion

## Simple APIClient example

```rust
use http_adapter::http::Request;
use http_adapter::HttpClientAdapter;

struct APIClient<HttpClient> {
    http_client: HttpClient,
}

impl<HttpClient: HttpClientAdapter> APIClient<HttpClient> {
    /// Create new `APIClient` by supplying an HTTP client implementation
    pub fn new(http_client: HttpClient) -> Self {
        Self { http_client }
    }

    pub async fn create_entry(&self) -> Result<(), HttpClient::Error> {
        let request = Request::post("http://localhost")
            .header(http::header::AUTHORIZATION, "Bearer 12345")
            .body(r#"{ "value": 42 }"#.as_bytes().to_vec())
            .expect("Can't create request");
        let response = self.http_client.execute(request).await?;
        Ok(())
    }
}

/// Default implementation for cases where adapter implements `Default`
impl<HttpClient: HttpClientAdapter + Default> Default for APIClient<HttpClient> {
    fn default() -> Self {
        Self::new(HttpClient::default())
    }
}
```

[1]: <https://crates.io/crates/http-adapter-reqwest>
[2]: <https://crates.io/crates/http-adapter-surf>
[3]: <https://crates.io/crates/http-adapter-ureq>

License: LGPL-3.0
