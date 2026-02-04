// crates.io
use ws_tool::{
	ClientBuilder, ServerBuilder,
	codec::{self, AsyncBytesCodec},
};
// self
use crate::{
	bench::{BenchConfig, Mode, duration_to_micros, report_latency, report_throughput},
	prelude::*,
	set_nodelay,
};

use color_eyre::eyre::{Result, WrapErr, eyre};

pub(super) async fn run_server(listener: TcpListener) -> Result<()> {
	let (stream, _addr) = listener.accept().await?;
	set_nodelay(&stream)?;

	let (mut rx, mut tx) = ServerBuilder::async_accept(
		stream,
		codec::default_handshake_handler,
		AsyncBytesCodec::factory,
	)
	.await?
	.split();

	loop {
		let msg = rx.receive().await?;

		if msg.code.is_close() {
			break;
		} else {
			tx.send(msg).await?;
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
	let uri = format!("ws://{addr}").try_into().wrap_err("Failed to parse websocket URI.")?;
	let stream = TcpStream::connect(addr).await?;

	set_nodelay(&stream)?;

	let (mut rx, mut tx) = ClientBuilder::new()
		.async_with_stream(uri, stream, AsyncBytesCodec::check_fn)
		.await?
		.split();

	match mode {
		Mode::Rtt => run_rtt(&mut rx, &mut tx, config, payload).await?,
		Mode::Throughput => run_throughput(&mut rx, &mut tx, config, payload).await?,
	}

	tx.send((ws_tool::frame::OpCode::Close, &[])).await?;
	Ok(())
}

async fn run_rtt<R, W>(
	rx: &mut ws_tool::codec::AsyncBytesRecv<R>,
	tx: &mut ws_tool::codec::AsyncBytesSend<W>,
	config: &BenchConfig,
	payload: &[u8],
) -> Result<()>
where
	R: tokio::io::AsyncRead + Unpin,
	W: tokio::io::AsyncWrite + Unpin,
{
	let mut samples = Vec::with_capacity(config.rounds as usize);

	for round in 0..(config.warmup_rounds + config.rounds) {
		let start = Instant::now();
		for _ in 0..config.msg_count {
			tx.send(payload).await?;
			let _msg = rx.receive().await?;
		}
		let elapsed = start.elapsed();
		if round >= config.warmup_rounds {
			let micros = duration_to_micros(elapsed) / config.msg_count as f64;
			samples.push(micros);
		}
	}

	report_latency(&samples);
	Ok(())
}

async fn run_throughput<R, W>(
	rx: &mut ws_tool::codec::AsyncBytesRecv<R>,
	tx: &mut ws_tool::codec::AsyncBytesSend<W>,
	config: &BenchConfig,
	payload: &[u8],
) -> Result<()>
where
	R: tokio::io::AsyncRead + Unpin,
	W: tokio::io::AsyncWrite + Unpin,
{
	let mut samples = Vec::with_capacity(config.rounds as usize);

	for round in 0..(config.warmup_rounds + config.rounds) {
		let start = Instant::now();
		let send = async {
			for _ in 0..config.msg_count {
				tx.send(payload).await?;
			}
			Ok::<(), color_eyre::Report>(())
		};
		let recv = async {
			for _ in 0..config.msg_count {
				let msg = rx.receive().await?;
				if msg.code.is_close() {
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
	Ok(())
}
