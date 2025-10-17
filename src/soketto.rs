// crates.io
use futures::io::{BufReader, BufWriter};
use soketto::{
	Data, Incoming,
	handshake::{Client, Server, server::Response},
};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncReadCompatExt;
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
			let mut server = Server::new(BufReader::new(BufWriter::new(stream.compat())));
			let key = server.receive_request().await.unwrap().key();
			let accept = Response::Accept { key, protocol: None };

			server.send_response(&accept).await.unwrap();

			let (mut tx, mut rx) = server.into_builder().finish();

			println!("server: WS handshake successful");

			// Echo loop.
			loop {
				let mut msg = Vec::new();

				match rx.receive(&mut msg).await.unwrap() {
					Incoming::Data(Data::Binary(_)) => {
						tx.send_binary_mut(&mut msg).await.unwrap();
						tx.flush().await.unwrap();
					},
					Incoming::Closed(_) => {
						println!("server: received close signal");

						break;
					},
					_ => unreachable!(),
				}
			}
		});
	});

	server_ready_rx.await.unwrap();

	let client_handle: tokio::task::JoinHandle<()> = tokio::spawn(async move {
		let stream = TcpStream::connect(ADDR).await.unwrap();
		let mut ws_stream = Client::new(BufReader::new(BufWriter::new(stream.compat())), ADDR, "/");

		ws_stream.handshake().await.unwrap();

		let (mut tx, mut rx) = ws_stream.into_builder().finish();

		println!("client: WS connected");

		let mut duration = Duration::default();

		for _i in 0..MSG_COUNT {
			let start = Instant::now();
			let mut msg = Vec::new();

			tx.send_binary(PAYLOAD).await.unwrap();
			tx.flush().await.unwrap();

			let _msg = rx.receive(&mut msg).await.unwrap();
			let elapsed = start.elapsed();

			duration += elapsed;

			// println!("client: round trip #{_i} took {elapsed:.2?}");
		}

		let duration_avg = duration / MSG_COUNT;

		println!("client: sent {MSG_COUNT} messages, average round trip time {duration_avg:.2?}");

		tx.close().await.unwrap();
	});

	client_handle.await.unwrap();
	server_handle.await.unwrap();
}
