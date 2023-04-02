use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};

pub use surf;
use surf::http::url::ParseError;
use surf::http::{Method as SurfMethod, StatusCode as SurfStatusCode, Version as SurfVersion};

use http_adapter::async_trait::async_trait;
use http_adapter::http::{Method, StatusCode, Version};
use http_adapter::{http, HttpClientAdapter};
use http_adapter::{Request, Response};

#[derive(Clone, Debug)]
pub struct SurfAdapter {
	client: surf::Client,
}

impl SurfAdapter {
	pub fn new(client: surf::Client) -> Self {
		Self { client }
	}
}

impl Default for SurfAdapter {
	#[inline]
	fn default() -> Self {
		Self {
			client: surf::Client::new(),
		}
	}
}

#[derive(Debug)]
pub enum Error {
	Http(http::Error),
	Surf(surf::Error),
	InvalidMethod(String),
	InvalidStatusCode(u16),
	InvalidHttpVersion(String),
	InvalidHeader(String, Vec<u8>),
	InvalidUrl(ParseError),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			Error::Http(e) => Display::fmt(e, f),
			Error::Surf(e) => Display::fmt(e, f),
			Error::InvalidStatusCode(code) => {
				write!(f, "Invalid status code: {code}")
			}
			Error::InvalidMethod(method) => {
				write!(f, "Invalid method: {method}")
			}
			Error::InvalidHttpVersion(version) => {
				write!(f, "Invalid HTTP version: {version}")
			}
			Error::InvalidHeader(name, value) => {
				write!(f, "Invalid header: {name} value: {value:?}")
			}
			Error::InvalidUrl(e) => {
				write!(f, "Invalid URL: {e}")
			}
		}
	}
}

impl StdError for Error {}

#[inline]
fn from_request(request: Request<Vec<u8>>) -> Result<surf::Request, Error> {
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
		_ => {
			return Err(Error::InvalidMethod(head.method.to_string()));
		}
	};
	let mut out = surf::Request::new(method, surf::Url::parse(&head.uri.to_string()).map_err(Error::InvalidUrl)?);
	for (name, value) in &head.headers {
		out.append_header(
			name.as_str(),
			value
				.to_str()
				.map_err(|_| Error::InvalidHeader(name.to_string(), value.as_bytes().to_vec()))?,
		);
	}
	out.body_bytes(body);
	Ok(out)
}

#[inline]
async fn to_response(mut res: surf::Response) -> Result<Response<Vec<u8>>, Error> {
	let status = match res.status() {
		SurfStatusCode::Continue => StatusCode::CONTINUE,
		SurfStatusCode::SwitchingProtocols => StatusCode::SWITCHING_PROTOCOLS,
		SurfStatusCode::Ok => StatusCode::OK,
		SurfStatusCode::Created => StatusCode::CREATED,
		SurfStatusCode::Accepted => StatusCode::ACCEPTED,
		SurfStatusCode::NonAuthoritativeInformation => StatusCode::NON_AUTHORITATIVE_INFORMATION,
		SurfStatusCode::NoContent => StatusCode::NO_CONTENT,
		SurfStatusCode::ResetContent => StatusCode::RESET_CONTENT,
		SurfStatusCode::PartialContent => StatusCode::PARTIAL_CONTENT,
		SurfStatusCode::MultiStatus => StatusCode::MULTI_STATUS,
		SurfStatusCode::ImUsed => StatusCode::IM_USED,
		SurfStatusCode::MultipleChoice => StatusCode::MULTIPLE_CHOICES,
		SurfStatusCode::MovedPermanently => StatusCode::MOVED_PERMANENTLY,
		SurfStatusCode::Found => StatusCode::FOUND,
		SurfStatusCode::SeeOther => StatusCode::SEE_OTHER,
		SurfStatusCode::NotModified => StatusCode::NOT_MODIFIED,
		SurfStatusCode::TemporaryRedirect => StatusCode::TEMPORARY_REDIRECT,
		SurfStatusCode::PermanentRedirect => StatusCode::PERMANENT_REDIRECT,
		SurfStatusCode::BadRequest => StatusCode::BAD_REQUEST,
		SurfStatusCode::Unauthorized => StatusCode::UNAUTHORIZED,
		SurfStatusCode::PaymentRequired => StatusCode::PAYMENT_REQUIRED,
		SurfStatusCode::Forbidden => StatusCode::FORBIDDEN,
		SurfStatusCode::NotFound => StatusCode::NOT_FOUND,
		SurfStatusCode::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
		SurfStatusCode::NotAcceptable => StatusCode::NOT_ACCEPTABLE,
		SurfStatusCode::ProxyAuthenticationRequired => StatusCode::PROXY_AUTHENTICATION_REQUIRED,
		SurfStatusCode::RequestTimeout => StatusCode::REQUEST_TIMEOUT,
		SurfStatusCode::Conflict => StatusCode::CONFLICT,
		SurfStatusCode::Gone => StatusCode::GONE,
		SurfStatusCode::LengthRequired => StatusCode::LENGTH_REQUIRED,
		SurfStatusCode::PreconditionFailed => StatusCode::PRECONDITION_FAILED,
		SurfStatusCode::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
		SurfStatusCode::UriTooLong => StatusCode::URI_TOO_LONG,
		SurfStatusCode::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
		SurfStatusCode::RequestedRangeNotSatisfiable => StatusCode::RANGE_NOT_SATISFIABLE,
		SurfStatusCode::ExpectationFailed => StatusCode::EXPECTATION_FAILED,
		SurfStatusCode::ImATeapot => StatusCode::IM_A_TEAPOT,
		SurfStatusCode::MisdirectedRequest => StatusCode::MISDIRECTED_REQUEST,
		SurfStatusCode::UnprocessableEntity => StatusCode::UNPROCESSABLE_ENTITY,
		SurfStatusCode::Locked => StatusCode::LOCKED,
		SurfStatusCode::FailedDependency => StatusCode::FAILED_DEPENDENCY,
		SurfStatusCode::UpgradeRequired => StatusCode::UPGRADE_REQUIRED,
		SurfStatusCode::PreconditionRequired => StatusCode::PRECONDITION_REQUIRED,
		SurfStatusCode::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
		SurfStatusCode::RequestHeaderFieldsTooLarge => StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE,
		SurfStatusCode::UnavailableForLegalReasons => StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS,
		SurfStatusCode::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
		SurfStatusCode::NotImplemented => StatusCode::NOT_IMPLEMENTED,
		SurfStatusCode::BadGateway => StatusCode::BAD_GATEWAY,
		SurfStatusCode::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
		SurfStatusCode::GatewayTimeout => StatusCode::GATEWAY_TIMEOUT,
		SurfStatusCode::HttpVersionNotSupported => StatusCode::HTTP_VERSION_NOT_SUPPORTED,
		SurfStatusCode::VariantAlsoNegotiates => StatusCode::VARIANT_ALSO_NEGOTIATES,
		SurfStatusCode::InsufficientStorage => StatusCode::INSUFFICIENT_STORAGE,
		SurfStatusCode::LoopDetected => StatusCode::LOOP_DETECTED,
		SurfStatusCode::NotExtended => StatusCode::NOT_EXTENDED,
		SurfStatusCode::NetworkAuthenticationRequired => StatusCode::NETWORK_AUTHENTICATION_REQUIRED,
		SurfStatusCode::EarlyHints | SurfStatusCode::TooEarly => return Err(Error::InvalidStatusCode(u16::from(res.status()))),
	};
	let mut response = Response::builder().status(status);

	if let Some(version) = res.version() {
		let version = match version {
			SurfVersion::Http0_9 => Version::HTTP_09,
			SurfVersion::Http1_0 => Version::HTTP_10,
			SurfVersion::Http1_1 => Version::HTTP_11,
			SurfVersion::Http2_0 => Version::HTTP_2,
			SurfVersion::Http3_0 => Version::HTTP_3,
			_ => return Err(Error::InvalidHttpVersion(version.to_string())),
		};
		response = response.version(version)
	}

	for header_name in res.header_names() {
		if let Some(header_values) = res.header(header_name) {
			for header_value in header_values {
				response = response.header(header_name.to_string(), header_value.to_string());
			}
		}
	}

	let body = res.body_bytes().await.map_err(Error::Surf)?;

	response.body(body).map_err(Error::Http)
}

#[async_trait(?Send)]
impl HttpClientAdapter for SurfAdapter {
	type Error = Error;

	async fn execute(&self, request: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, Self::Error> {
		let res = self.client.send(from_request(request)?).await.map_err(Error::Surf)?;
		to_response(res).await
	}
}
