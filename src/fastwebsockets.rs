// crates.io
use crate::prelude::*;
use bytes::Bytes;
use fastwebsockets::{FragmentCollector, Frame, OpCode, Payload, handshake};
use http_body_util::Empty;
use hyper::body::Incoming;
use hyper::header::{CONNECTION, UPGRADE};
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

struct SpawnExecutor;

impl<Fut> hyper::rt::Executor<Fut> for SpawnExecutor
where
	Fut: Future<Output = ()> + Send + 'static,
{
	fn execute(&self, fut: Fut) {
		tokio::spawn(fut);
	}
}

pub(super) async fn bench() {
	let (server_ready_tx, server_ready_rx) = oneshot::channel::<()>();
	let server_handle = tokio::spawn(async move {
		let listener = TcpListener::bind(ADDR).await.unwrap();

		println!("server: listening on {ADDR}");

		server_ready_tx.send(()).unwrap();

		let (stream, addr) = listener.accept().await.unwrap();

		println!("server: new connection from {addr}");

		let io = TokioIo::new(stream);

		let service = hyper::service::service_fn(move |mut req: Request<Incoming>| async move {
			let (response, fut) = fastwebsockets::upgrade::upgrade(&mut req)?;

			tokio::spawn(async move {
				let mut ws = FragmentCollector::new(fut.await.unwrap());
				loop {
					let frame = match ws.read_frame().await {
						Ok(frame) => frame,
						Err(_) => break,
					};

					match frame.opcode {
						OpCode::Close => break,
						OpCode::Text | OpCode::Binary => {
							if ws.write_frame(frame).await.is_err() {
								break;
							}
						},
						_ => {},
					}
				}
			});

			Ok::<Response<Empty<Bytes>>, fastwebsockets::WebSocketError>(response)
		});

		if let Err(err) = hyper::server::conn::http1::Builder::new()
			.serve_connection(io, service)
			.with_upgrades()
			.await
		{
			eprintln!("server: error serving connection: {err:?}");
		}
	});

	server_ready_rx.await.unwrap();

	let client_handle = tokio::spawn(async move {
		let stream = TcpStream::connect(ADDR).await.unwrap();

		let req = Request::builder()
			.method("GET")
			.uri(format!("ws://{ADDR}/"))
			.header(UPGRADE, "websocket")
			.header(CONNECTION, "upgrade")
			.header("Sec-WebSocket-Key", handshake::generate_key())
			.header("Sec-WebSocket-Version", "13")
			.body(Empty::<Bytes>::new())
			.unwrap();

		let (ws, _) = handshake::client(&SpawnExecutor, req, stream).await.unwrap();
		let mut ws = FragmentCollector::new(ws);

		println!("client: WS connected");

		let mut duration = Duration::default();

		for _ in 0..MSG_COUNT {
			let start = Instant::now();

			ws.write_frame(Frame::binary(Payload::Borrowed(PAYLOAD))).await.unwrap();

			let _frame = ws.read_frame().await.unwrap();
			let elapsed = start.elapsed();

			duration += elapsed;
		}

		let duration_avg = duration / MSG_COUNT;

		println!("client: sent {MSG_COUNT} messages, average round trip time {duration_avg:.2?}");

		ws.write_frame(Frame::close(1000, &[])).await.unwrap();
	});

	client_handle.await.unwrap();
	server_handle.await.unwrap();
}
