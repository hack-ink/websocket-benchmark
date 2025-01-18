// crates.io
use ws_tool::{
	codec::{self, AsyncBytesCodec},
	ClientBuilder, ServerBuilder,
};
// self
use crate::prelude::*;

pub(super) async fn bench() {
	let (server_ready_tx, server_ready_rx) = oneshot::channel::<()>();
	let server_handle = tokio::spawn(async move {
		let listener = TcpListener::bind(ADDR).await.unwrap();

		println!("server: listening on {ADDR}");

		server_ready_tx.send(()).unwrap();

		let (stream, addr) = listener.accept().await.unwrap();

		println!("server: new connection from {addr}");

		// Spawn a new task to handle the connection.
		tokio::spawn(async move {
			let (mut rx, mut tx) = ServerBuilder::async_accept(
				stream,
				codec::default_handshake_handler,
				AsyncBytesCodec::factory,
			)
			.await
			.unwrap()
			.split();

			println!("server: WS handshake successful");

			// Echo loop.
			loop {
				let msg = rx.receive().await.unwrap();

				if msg.code.is_close() {
					println!("server: received close signal");

					break;
				} else {
					// Echo the message back.
					if let Err(e) = tx.send(msg).await {
						eprintln!("server: failed to echo msg, error: {e}");

						break;
					}
				}
			}
		});
	});

	server_ready_rx.await.unwrap();

	let client_handle = tokio::spawn(async move {
		let (mut rx, mut tx) = ClientBuilder::new()
			.async_connect(format!("ws://{ADDR}").try_into().unwrap(), AsyncBytesCodec::check_fn)
			.await
			.unwrap()
			.split();

		println!("client: WS connected");

		let mut duration = Duration::default();

		for _i in 0..MSG_COUNT {
			let start = Instant::now();

			tx.send(PAYLOAD).await.unwrap();

			let _msg = rx.receive().await.unwrap();
			let elapsed = start.elapsed();

			duration += elapsed;

			// println!("client: round trip #{_i} took {elapsed:.2?}");
		}

		let duration_avg = duration / MSG_COUNT;

		println!("client: sent {MSG_COUNT} messages, average round trip time {duration_avg:.2?}");

		tx.send((ws_tool::frame::OpCode::Close, &[])).await.unwrap();
	});

	client_handle.await.unwrap();
	server_handle.await.unwrap();
}
