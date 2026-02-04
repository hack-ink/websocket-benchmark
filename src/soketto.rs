// crates.io
use futures::io::{AsyncRead, AsyncWrite, BufReader, BufWriter};
use soketto::{
	Data, Incoming,
	handshake::{Client, Server, server::Response},
};
use tokio_util::compat::TokioAsyncReadCompatExt;
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

	let mut server = Server::new(BufReader::new(BufWriter::new(stream.compat())));
	let key = server.receive_request().await?.key();
	let accept = Response::Accept { key, protocol: None };
	server.send_response(&accept).await?;

	let (mut tx, mut rx) = server.into_builder().finish();
	let mut msg = Vec::new();

	loop {
		msg.clear();
		match rx.receive(&mut msg).await? {
			Incoming::Data(Data::Binary(_)) => {
				tx.send_binary_mut(&mut msg).await?;
				tx.flush().await?;
			},
			Incoming::Closed(_) => break,
			_ => return Err(eyre!("Unexpected message type received on the server.")),
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

	let host = addr.to_string();
	let mut ws_stream =
		Client::new(BufReader::new(BufWriter::new(stream.compat())), host.as_str(), "/");
	ws_stream.handshake().await?;

	let (mut tx, mut rx) = ws_stream.into_builder().finish();

	match mode {
		Mode::Rtt => run_rtt(&mut tx, &mut rx, config, payload).await?,
		Mode::Throughput => run_throughput(&mut tx, &mut rx, config, payload).await?,
	}

	tx.close().await?;
	Ok(())
}

async fn run_rtt<W, R>(
	tx: &mut soketto::Sender<W>,
	rx: &mut soketto::Receiver<R>,
	config: &BenchConfig,
	payload: &[u8],
) -> Result<()>
where
	W: AsyncRead + AsyncWrite + Unpin,
	R: AsyncRead + AsyncWrite + Unpin,
{
	let mut samples = Vec::with_capacity(config.rounds as usize);
	let mut msg = Vec::with_capacity(config.payload_len);

	for round in 0..(config.warmup_rounds + config.rounds) {
		let start = Instant::now();
		for _ in 0..config.msg_count {
			tx.send_binary(payload).await?;
			tx.flush().await?;

			msg.clear();
			match rx.receive(&mut msg).await? {
				Incoming::Data(Data::Binary(_)) => {},
				Incoming::Closed(_) =>
					return Err(eyre!("Server closed the connection during RTT.")),
				_ => return Err(eyre!("Unexpected message type received during RTT.")),
			}
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

async fn run_throughput<W, R>(
	tx: &mut soketto::Sender<W>,
	rx: &mut soketto::Receiver<R>,
	config: &BenchConfig,
	payload: &[u8],
) -> Result<()>
where
	W: AsyncRead + AsyncWrite + Unpin,
	R: AsyncRead + AsyncWrite + Unpin,
{
	let mut samples = Vec::with_capacity(config.rounds as usize);

	for round in 0..(config.warmup_rounds + config.rounds) {
		let start = Instant::now();
		let send = async {
			for _ in 0..config.msg_count {
				tx.send_binary(payload).await?;
				tx.flush().await?;
			}
			Ok::<(), color_eyre::Report>(())
		};
		let recv = async {
			let mut msg = Vec::with_capacity(config.payload_len);
			for _ in 0..config.msg_count {
				msg.clear();
				match rx.receive(&mut msg).await? {
					Incoming::Data(Data::Binary(_)) => {},
					Incoming::Closed(_) =>
						return Err(eyre!("Server closed the connection during throughput.")),
					_ => return Err(eyre!("Unexpected message type received during throughput.")),
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
