//! # HTTP adapter implementation for [ureq](https://crates.io/crates/ureq)
//!
//! For more details refer to [http-adapter](https://crates.io/crates/http-adapter)

use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::future::Future;
use std::io::Read;
use std::pin::Pin;
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::task::{Context, Poll, Waker};
use std::thread::JoinHandle;
use std::{io, thread};

pub use ureq;

use http_adapter::async_trait::async_trait;
use http_adapter::http::HeaderValue;
use http_adapter::{http, HttpClientAdapter};
use http_adapter::{Request, Response};

#[derive(Clone, Debug)]
pub struct UreqAdapter {
	agent: ureq::Agent,
}

impl UreqAdapter {
	pub fn new(agent: ureq::Agent) -> Self {
		Self { agent }
	}
}

impl Default for UreqAdapter {
	#[inline]
	fn default() -> Self {
		Self {
			agent: ureq::Agent::new_with_defaults(),
		}
	}
}

#[derive(Debug)]
pub enum Error {
	Http(http::Error),
	Ureq(ureq::Error),
	Io(io::Error),
	InvalidHeaderValue(HeaderValue),
	InvalidHttpVersion(String),
	InvalidStatusCode(u16),
	InternalCommunicationError(String),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> FmtResult {
		match self {
			Error::Http(e) => Display::fmt(e, f),
			Error::Ureq(e) => Display::fmt(e, f),
			Error::Io(e) => Display::fmt(e, f),
			Error::InvalidHeaderValue(header_value) => {
				write!(f, "Invalid header value: {header_value:?}")
			}
			Error::InvalidHttpVersion(version) => {
				write!(f, "Invalid HTTP version: {version}")
			}
			Error::InvalidStatusCode(code) => {
				write!(f, "Invalid status code: {code}")
			}
			Error::InternalCommunicationError(e) => {
				write!(f, "Internal communication error: {e}")
			}
		}
	}
}

impl StdError for Error {}

#[inline]
fn to_response(res: Response<ureq::Body>) -> Result<Response<Vec<u8>>, Error> {
	let mut body_read_err = None;
	let new_resp = res.map(|body| {
		let mut out = Vec::with_capacity(usize::try_from(body.content_length().unwrap_or(4 * 1024)).unwrap_or(4 * 1024));
		let res = body.into_reader().read_to_end(&mut out);
		match res {
			Ok(_) => out,
			Err(err) => {
				body_read_err = Some(err);
				Vec::new()
			}
		}
	});
	match body_read_err {
		None => Ok(new_resp),
		Some(body_read_err) => Err(Error::Io(body_read_err)),
	}
}

#[async_trait]
impl HttpClientAdapter for UreqAdapter {
	type Error = Error;

	async fn execute(&self, request: Request<Vec<u8>>) -> Result<Response<Vec<u8>>, Self::Error> {
		let agent = self.agent.clone();
		let res = ThreadFuture::new(|send_result, recv_waker| {
			move || {
				let waker = recv_waker
					.recv()
					.map_err(|_| Error::InternalCommunicationError("Waker receive channel is closed".to_string()))?;
				match agent.run(request).map_err(Error::Ureq) {
					Ok(res) => send_result
						.send(to_response(res))
						.map_err(|_| Error::InternalCommunicationError("Result send channel is closed for Ok result".to_string()))?,
					Err(e) => send_result
						.send(Err(e))
						.map_err(|_| Error::InternalCommunicationError("Result send channel is closed for Err result".to_string()))?,
				}
				waker.wake();
				Ok(())
			}
		})
		.await;
		match res {
			FutureResult::CommunicationError(e) => Err(Error::InternalCommunicationError(e)),
			FutureResult::Result(r) => r,
		}
	}
}

struct ThreadFuture<Res> {
	thread: Option<JoinHandle<Result<(), Error>>>,
	recv_result: Receiver<Res>,
	send_waker: Sender<Waker>,
	waker_sent: bool,
}

impl<Res: Send + 'static> ThreadFuture<Res> {
	pub fn new<Factory, Body>(factory: Factory) -> ThreadFuture<Res>
	where
		Factory: FnOnce(Sender<Res>, Receiver<Waker>) -> Body,
		Body: FnOnce() -> Result<(), Error> + Send + 'static,
	{
		let (send_result, recv_result) = channel();
		let (send_waker, recv_waker) = channel();
		let body = factory(send_result, recv_waker);
		let thread = thread::spawn(body);
		ThreadFuture {
			thread: Some(thread),
			recv_result,
			send_waker,
			waker_sent: false,
		}
	}
}

impl<Res> Drop for ThreadFuture<Res> {
	fn drop(&mut self) {
		if let Some(thread) = self.thread.take() {
			let _ = thread.join().expect("Can't join thread");
		}
	}
}

impl<Res> Future for ThreadFuture<Res> {
	type Output = FutureResult<Res>;

	fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		if !self.waker_sent {
			if self.send_waker.send(cx.waker().clone()).is_err() {
				return Poll::Ready(FutureResult::CommunicationError("Waker send channel is closed".to_string()));
			}
			self.waker_sent = true;
		}
		match self.recv_result.try_recv() {
			Ok(res) => Poll::Ready(FutureResult::Result(res)),
			Err(TryRecvError::Disconnected) => Poll::Ready(FutureResult::CommunicationError(
				"Result receive channel is closed".to_string(),
			)),
			Err(TryRecvError::Empty) => Poll::Pending,
		}
	}
}

enum FutureResult<Res> {
	CommunicationError(String),
	Result(Res),
}
