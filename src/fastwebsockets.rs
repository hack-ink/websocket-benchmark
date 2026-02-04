// crates.io
use fastwebsockets::{FragmentCollector, FragmentCollectorRead, Frame, OpCode, handshake, upgrade};
use http_body_util::Empty;
use hyper::{
	Request,
	body::{Bytes, Incoming},
	header::{CONNECTION, UPGRADE},
	server::conn::http1,
	service::service_fn,
};
use hyper_util::rt::TokioIo;
// self
use crate::{
	bench::{BenchConfig, Mode, duration_to_micros, report_latency, report_throughput},
	prelude::*,
	set_nodelay,
};

use color_eyre::eyre::{Result, eyre};
use std::{
	future::Future,
	sync::{Arc, Mutex},
};

pub(super) async fn run_server(listener: TcpListener) -> Result<()> {
	let (stream, _addr) = listener.accept().await?;
	set_nodelay(&stream)?;

	let io = TokioIo::new(stream);
	let (upgrade_tx, upgrade_rx) = tokio::sync::oneshot::channel();
	let upgrade_tx = Arc::new(Mutex::new(Some(upgrade_tx)));
	let service = {
		let upgrade_tx = Arc::clone(&upgrade_tx);
		service_fn(move |mut req: Request<Incoming>| {
			let upgrade_tx = Arc::clone(&upgrade_tx);
			async move {
				let (response, fut) = upgrade::upgrade(&mut req)?;
				let mut guard =
					upgrade_tx.lock().map_err(|_| eyre!("Upgrade channel lock was poisoned."))?;
				if let Some(tx) = guard.take() {
					let _ = tx.send(fut);
				}
				Ok::<_, color_eyre::Report>(response)
			}
		})
	};

	let connection = http1::Builder::new().serve_connection(io, service).with_upgrades();
	let upgrade = async {
		let fut = upgrade_rx
			.await
			.map_err(|_| eyre!("Upgrade channel closed before handshake completed."))?;
		handle_connection(fut).await?;
		Ok::<(), color_eyre::Report>(())
	};

	let (connection_result, upgrade_result) = tokio::join!(connection, upgrade);
	connection_result?;
	upgrade_result?;
	Ok(())
}

async fn handle_connection(fut: upgrade::UpgradeFut) -> Result<(), fastwebsockets::WebSocketError> {
	let mut ws = FragmentCollector::new(fut.await?);

	loop {
		let frame = ws.read_frame().await?;
		match frame.opcode {
			OpCode::Close => break,
			OpCode::Text | OpCode::Binary => {
				let outgoing = Frame::new(frame.fin, frame.opcode, None, frame.payload);
				ws.write_frame(outgoing).await?;
			},
			_ => {},
		}
	}

	Ok(())
}

pub(super) async fn run_client(
	addr: SocketAddr,
	config: &BenchConfig,
	mode: Mode,
	payload: &[u8],
) -> Result<()> {
	let ws = connect(addr).await?;

	match mode {
		Mode::Rtt => run_rtt(ws, config, payload).await?,
		Mode::Throughput => run_throughput(ws, config, payload).await?,
	}

	Ok(())
}

async fn connect(
	addr: SocketAddr,
) -> Result<fastwebsockets::WebSocket<TokioIo<hyper::upgrade::Upgraded>>> {
	let stream = TcpStream::connect(addr).await?;
	set_nodelay(&stream)?;

	let uri: hyper::Uri = format!("http://{addr}/").parse()?;
	let req = Request::builder()
		.method("GET")
		.uri(uri)
		.header("Host", addr.to_string())
		.header(UPGRADE, "websocket")
		.header(CONNECTION, "upgrade")
		.header("Sec-WebSocket-Key", handshake::generate_key())
		.header("Sec-WebSocket-Version", "13")
		.body(Empty::<Bytes>::new())?;

	let (mut ws, _response) = handshake::client(&SpawnExecutor, req, stream).await?;
	ws.set_auto_close(true);
	ws.set_auto_pong(true);
	Ok(ws)
}

async fn run_rtt(
	ws: fastwebsockets::WebSocket<TokioIo<hyper::upgrade::Upgraded>>,
	config: &BenchConfig,
	payload: &[u8],
) -> Result<()> {
	let mut samples = Vec::with_capacity(config.rounds as usize);
	let mut ws = FragmentCollector::new(ws);

	for round in 0..(config.warmup_rounds + config.rounds) {
		let start = Instant::now();
		for _ in 0..config.msg_count {
			let frame = Frame::binary(payload.to_vec().into());
			ws.write_frame(frame).await?;
			let frame = ws.read_frame().await?;
			if frame.opcode == OpCode::Close {
				return Err(eyre!("Server closed the connection during RTT."));
			}
		}
		let elapsed = start.elapsed();
		if round >= config.warmup_rounds {
			let micros = duration_to_micros(elapsed) / config.msg_count as f64;
			samples.push(micros);
		}
	}

	report_latency(&samples);
	ws.write_frame(Frame::close(1000, b"")).await?;
	Ok(())
}

async fn run_throughput(
	ws: fastwebsockets::WebSocket<TokioIo<hyper::upgrade::Upgraded>>,
	config: &BenchConfig,
	payload: &[u8],
) -> Result<()> {
	let mut samples = Vec::with_capacity(config.rounds as usize);
	let (mut reader, mut writer) = ws.split(tokio::io::split);
	reader.set_auto_close(false);
	reader.set_auto_pong(false);
	let mut reader = FragmentCollectorRead::new(reader);

	for round in 0..(config.warmup_rounds + config.rounds) {
		let start = Instant::now();
		let send = async {
			for _ in 0..config.msg_count {
				let frame = Frame::binary(payload.to_vec().into());
				writer.write_frame(frame).await?;
			}
			Ok::<(), color_eyre::Report>(())
		};
		let recv = async {
			let mut send_fn = |_frame| async { Ok::<(), std::io::Error>(()) };
			for _ in 0..config.msg_count {
				let frame = reader.read_frame(&mut send_fn).await?;
				if frame.opcode == OpCode::Close {
					return Err(eyre!("Server closed the connection during throughput."));
				}
			}
			Ok::<(), color_eyre::Report>(())
		};

		let (send_result, recv_result) = tokio::join!(send, recv);
		send_result?;
		recv_result?;

		let elapsed = start.elapsed();
		if round >= config.warmup_rounds {
			let bytes = config.msg_count as f64 * config.payload_len as f64 * 2.0;
			let mib_per_sec = bytes / (1024.0 * 1024.0) / elapsed.as_secs_f64();
			samples.push(mib_per_sec);
		}
	}

	report_throughput(&samples);
	writer.write_frame(Frame::close(1000, b"")).await?;
	writer.flush().await?;
	Ok(())
}

struct SpawnExecutor;

impl<Fut> hyper::rt::Executor<Fut> for SpawnExecutor
where
	Fut: Future + Send + 'static,
	Fut::Output: Send + 'static,
{
	fn execute(&self, fut: Fut) {
		tokio::task::spawn(fut);
	}
}
