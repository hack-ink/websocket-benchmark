// crates.io
use tokio_websockets::{ClientBuilder, Message, ServerBuilder};
// self
use crate::{
	bench::{BenchConfig, Mode, duration_to_micros, report_latency, report_throughput},
	prelude::*,
	set_nodelay,
};

use color_eyre::eyre::{Result, eyre};

pub(super) async fn run_server(listener: TcpListener) -> Result<()> {
	let (stream, _addr) = listener.accept().await?;
	set_nodelay(&stream)?;

	let (_, mut ws_stream) = ServerBuilder::new().accept(stream).await?;

	while let Some(maybe_msg) = ws_stream.next().await {
		let msg = maybe_msg?;

		if msg.is_close() {
			break;
		} else {
			ws_stream.send(msg).await?;
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
	let stream = TcpStream::connect(addr).await?;
	set_nodelay(&stream)?;

	let url = format!("ws://{addr}");
	let (ws_stream, _) = ClientBuilder::new().uri(&url)?.connect_on(stream).await?;

	match mode {
		Mode::Rtt => run_rtt(ws_stream, config, payload).await?,
		Mode::Throughput => run_throughput(ws_stream, config, payload).await?,
	}

	Ok(())
}

async fn run_rtt(
	mut ws_stream: tokio_websockets::WebSocketStream<TcpStream>,
	config: &BenchConfig,
	payload: &[u8],
) -> Result<()> {
	let mut samples = Vec::with_capacity(config.rounds as usize);

	for round in 0..(config.warmup_rounds + config.rounds) {
		let start = Instant::now();
		for _ in 0..config.msg_count {
			ws_stream.send(Message::binary(payload.to_vec())).await?;
			let msg = ws_stream
				.next()
				.await
				.ok_or_else(|| eyre!("Server closed the connection during RTT."))??;
			if msg.is_close() {
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
	ws_stream.close().await?;
	Ok(())
}

async fn run_throughput(
	ws_stream: tokio_websockets::WebSocketStream<TcpStream>,
	config: &BenchConfig,
	payload: &[u8],
) -> Result<()> {
	let mut samples = Vec::with_capacity(config.rounds as usize);
	let (mut sink, mut stream) = ws_stream.split();

	for round in 0..(config.warmup_rounds + config.rounds) {
		let start = Instant::now();
		let send = async {
			for _ in 0..config.msg_count {
				sink.send(Message::binary(payload.to_vec())).await?;
			}
			Ok::<(), color_eyre::Report>(())
		};
		let recv = async {
			for _ in 0..config.msg_count {
				let msg = stream
					.next()
					.await
					.ok_or_else(|| eyre!("Server closed the connection during throughput."))??;
				if msg.is_close() {
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
	sink.send(Message::close(None, "")).await?;
	Ok(())
}
