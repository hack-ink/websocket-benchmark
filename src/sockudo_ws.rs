use crate::prelude::*;
use bytes::Bytes;
use sockudo_ws::{Config, Message, WebSocketStream, error::CloseReason, handshake};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub(super) async fn bench() {
	let (server_ready_tx, server_ready_rx) = oneshot::channel::<()>();
	let server_handle = tokio::spawn(async move {
		let listener = TcpListener::bind(ADDR).await.unwrap();

		println!("server: listening on {ADDR}");

		server_ready_tx.send(()).unwrap();

		let (mut stream, addr) = listener.accept().await.unwrap();

		println!("server: new connection from {addr}");

		let mut buf = [0u8; 1024];
		let n = stream.read(&mut buf).await.unwrap();
		let (request, _) = handshake::parse_request(&buf[..n]).unwrap().unwrap();
		let accept_key = handshake::generate_accept_key(request.key);
		let response = handshake::build_response(&accept_key, None, None);
		stream.write_all(&response).await.unwrap();

		println!("server: WS handshake successful");

		let mut ws = WebSocketStream::server(stream, Config::default());

		// Echo loop.
		while let Some(msg) = ws.next().await {
			let msg = msg.unwrap();

			if msg.is_close() {
				println!("server: received close signal");

				break;
			} else {
				// Echo the message back.
				if let Err(e) = ws.send(msg).await {
					eprintln!("server: failed to echo msg, error: {e}");

					break;
				}
			}
		}
	});

	server_ready_rx.await.unwrap();

	let client_handle = tokio::spawn(async move {
		let mut stream = TcpStream::connect(ADDR).await.unwrap();

		let key = handshake::generate_key();
		let request = format!(
			"GET / HTTP/1.1\r\n\
			 Host: {ADDR}\r\n\
			 Upgrade: websocket\r\n\
			 Connection: Upgrade\r\n\
			 Sec-WebSocket-Key: {key}\r\n\
			 Sec-WebSocket-Version: 13\r\n\r\n"
		);
		stream.write_all(request.as_bytes()).await.unwrap();

		let mut buf = [0u8; 1024];
		let _n = stream.read(&mut buf).await.unwrap();

		let mut ws = WebSocketStream::client(stream, Config::default());

		println!("client: WS connected");

		let mut duration = Duration::default();
		let payload = Bytes::from_static(PAYLOAD);

		for _i in 0..MSG_COUNT {
			let start = Instant::now();

			ws.send(Message::Binary(payload.clone())).await.unwrap();

			let _msg = ws.next().await.unwrap().unwrap();
			let elapsed = start.elapsed();

			duration += elapsed;
		}

		let duration_avg = duration / MSG_COUNT;

		println!("client: sent {MSG_COUNT} messages, average round trip time {duration_avg:.2?}");

		ws.send(Message::Close(Some(CloseReason { code: 1000, reason: "".into() }))).await.unwrap();
	});

	client_handle.await.unwrap();
	server_handle.await.unwrap();
}
