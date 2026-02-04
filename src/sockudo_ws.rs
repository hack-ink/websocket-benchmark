// crates.io
use sockudo_ws::{Config, Http1, Message, client::WebSocketClient, server::WebSocketServer};
// self
use crate::{
	bench::{BenchConfig, Mode, duration_to_micros, report_latency, report_throughput},
	prelude::*,
	set_nodelay,
};

use color_eyre::eyre::{Result, eyre};

type SockudoStream = sockudo_ws::WebSocketStream<sockudo_ws::Stream<Http1>>;

pub(super) async fn run_server(listener: TcpListener) -> Result<()> {
	let (stream, _addr) = listener.accept().await?;
	set_nodelay(&stream)?;

	let server = WebSocketServer::<Http1>::new(Config::default());
	let (mut ws, _handshake) = server.accept(stream).await?;

	while let Some(maybe_msg) = ws.next().await {
		let msg = maybe_msg?;

		if msg.is_close() {
			break;
		}

		ws.send(msg).await?;
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

	let host = addr.to_string();
	let client = WebSocketClient::<Http1>::new(Config::default());
	let (ws, _handshake) = client.connect(stream, &host, "/", None).await?;

	match mode {
		Mode::Rtt => run_rtt(ws, config, payload).await?,
		Mode::Throughput => run_throughput(ws, config, payload).await?,
	}

	Ok(())
}

async fn run_rtt(mut ws: SockudoStream, config: &BenchConfig, payload: &[u8]) -> Result<()> {
	let mut samples = Vec::with_capacity(config.rounds as usize);

	for round in 0..(config.warmup_rounds + config.rounds) {
		let start = Instant::now();
		for _ in 0..config.msg_count {
			ws.send(Message::binary(payload.to_vec())).await?;
			let msg = ws
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
	ws.close(1000, "").await?;
	Ok(())
}

async fn run_throughput(ws: SockudoStream, config: &BenchConfig, payload: &[u8]) -> Result<()> {
	let mut samples = Vec::with_capacity(config.rounds as usize);
	let (mut reader, mut writer) = ws.split();

	for round in 0..(config.warmup_rounds + config.rounds) {
		let start = Instant::now();
		let send = async {
			for _ in 0..config.msg_count {
				writer.send(Message::binary(payload.to_vec())).await?;
			}
			Ok::<(), color_eyre::Report>(())
		};
		let recv = async {
			for _ in 0..config.msg_count {
				let msg = reader
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
	writer.close(1000, "").await?;
	Ok(())
}
