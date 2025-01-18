// crates.io
use tokio_tungstenite::tungstenite::protocol::Message;
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
			let mut ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();

			println!("server: WS handshake successful");

			// Echo loop.
			while let Some(maybe_msg) = ws_stream.next().await {
				let msg = maybe_msg.unwrap();

				if msg.is_close() {
					println!("server: received close signal");

					break;
				} else {
					// Echo the message back.
					if let Err(e) = ws_stream.send(msg).await {
						eprintln!("server: failed to echo msg, error: {e}");

						break;
					}
				}
			}
		});
	});

	server_ready_rx.await.unwrap();

	let client_handle = tokio::spawn(async move {
		let (mut ws_stream, _) =
			tokio_tungstenite::connect_async(format!("ws://{ADDR}")).await.unwrap();

		println!("client: WS connected");

		let mut duration = Duration::default();

		for _i in 0..MSG_COUNT {
			let start = Instant::now();

			ws_stream.send(Message::binary(PAYLOAD)).await.unwrap();

			let _msg = ws_stream.next().await.unwrap().unwrap();
			let elapsed = start.elapsed();

			duration += elapsed;

			// println!("client: round trip #{_i} took {elapsed:.2?}");
		}

		let duration_avg = duration / MSG_COUNT;

		println!("client: sent {MSG_COUNT} messages, average round trip time {duration_avg:.2?}");

		ws_stream.close(None).await.unwrap();
	});

	client_handle.await.unwrap();
	server_handle.await.unwrap();
}
