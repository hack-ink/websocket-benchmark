//! Rust websocket benchmark.

#![deny(clippy::all, missing_docs, unused_crate_dependencies)]

mod bench;
mod fastwebsockets;
mod sockudo_ws;
mod soketto;
mod tokio_tungstenite;
mod tokio_websockets;
mod ws_tool;

mod prelude {
	pub use std::{net::SocketAddr, time::Instant};

	pub use futures::prelude::*;
	pub use tokio::net::{TcpListener, TcpStream};
}

use std::{
	io::{self, BufRead, Write},
	process::{Child, Command, Stdio},
	sync::mpsc,
	time::Duration,
};

use color_eyre::eyre::{Result, WrapErr, eyre};

use crate::bench::{BenchConfig, Mode};

const DEFAULT_MESSAGE_COUNT: u32 = 100_000;
const DEFAULT_PAYLOAD_SIZE: usize = 4_096;
const DEFAULT_WARMUP_ROUNDS: u32 = 1;
const DEFAULT_ROUNDS: u32 = 5;
const READY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy, Debug)]
enum Implementation {
	Fastwebsockets,
	SockudoWs,
	Soketto,
	TokioTungstenite,
	TokioWebsockets,
	WsTool,
}

impl Implementation {
	fn all() -> Vec<Self> {
		vec![
			Self::Soketto,
			Self::TokioTungstenite,
			Self::TokioWebsockets,
			Self::WsTool,
			Self::Fastwebsockets,
			Self::SockudoWs,
		]
	}

	fn as_str(self) -> &'static str {
		match self {
			Self::Fastwebsockets => "fastwebsockets",
			Self::SockudoWs => "sockudo-ws",
			Self::Soketto => "soketto",
			Self::TokioTungstenite => "tokio-tungstenite",
			Self::TokioWebsockets => "tokio-websockets",
			Self::WsTool => "ws-tool",
		}
	}

	fn parse(value: &str) -> Result<Self> {
		match value {
			"fastwebsockets" => Ok(Self::Fastwebsockets),
			"sockudo-ws" => Ok(Self::SockudoWs),
			"soketto" => Ok(Self::Soketto),
			"tokio-tungstenite" => Ok(Self::TokioTungstenite),
			"tokio-websockets" => Ok(Self::TokioWebsockets),
			"ws-tool" => Ok(Self::WsTool),
			_ => Err(eyre!(
				"Unknown implementation '{value}'. Use fastwebsockets, sockudo-ws, soketto, tokio-tungstenite, tokio-websockets, or ws-tool."
			)),
		}
	}
}

struct DriverConfig {
	impls: Vec<Implementation>,
	bench: BenchConfig,
}

struct ServerConfig {
	implementation: Implementation,
}

struct ClientConfig {
	implementation: Implementation,
	bench: BenchConfig,
	mode: Mode,
	addr: std::net::SocketAddr,
}

#[tokio::main]
async fn main() -> Result<()> {
	color_eyre::install().wrap_err("Failed to install error handler.")?;

	let args = std::env::args().skip(1).collect::<Vec<_>>();

	match parse_args(&args)? {
		ConfigUnion::Driver(config) => run_driver(config).wrap_err("Driver failed."),
		ConfigUnion::Server(config) => run_server(config).await.wrap_err("Server failed."),
		ConfigUnion::Client(config) => run_client(config).await.wrap_err("Client failed."),
	}
}

fn run_driver(config: DriverConfig) -> Result<()> {
	for implementation in config.impls {
		println!("Benchmarking {}.", implementation.as_str());

		run_phase(implementation, Mode::Rtt, &config.bench)?;
		run_phase(implementation, Mode::Throughput, &config.bench)?;

		println!();
	}

	Ok(())
}

fn run_phase(implementation: Implementation, mode: Mode, bench: &BenchConfig) -> Result<()> {
	let mut server = spawn_server(implementation)?;
	let port = match wait_for_ready(&mut server) {
		Ok(port) => port,
		Err(err) => {
			let _ = server.kill();
			return Err(err);
		},
	};

	match mode {
		Mode::Rtt => println!("Phase: RTT."),
		Mode::Throughput => println!("Phase: Throughput."),
	}

	let client_status = spawn_client(implementation, mode, bench, port)?.wait()?;
	if !client_status.success() {
		let _ = server.kill();
		return Err(eyre!("Client exited with status {client_status}."));
	}

	let server_status = server.wait()?;
	if !server_status.success() {
		return Err(eyre!("Server exited with status {server_status}."));
	}

	Ok(())
}

async fn run_server(config: ServerConfig) -> Result<()> {
	let addr = std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 0);
	let listener = tokio::net::TcpListener::bind(addr).await?;
	let port = listener.local_addr()?.port();

	println!("Ready on port {port}.");
	io::stdout().flush().wrap_err("Failed to flush readiness output.")?;

	match config.implementation {
		Implementation::Fastwebsockets => fastwebsockets::run_server(listener).await,
		Implementation::SockudoWs => sockudo_ws::run_server(listener).await,
		Implementation::Soketto => soketto::run_server(listener).await,
		Implementation::TokioTungstenite => tokio_tungstenite::run_server(listener).await,
		Implementation::TokioWebsockets => tokio_websockets::run_server(listener).await,
		Implementation::WsTool => ws_tool::run_server(listener).await,
	}
}

async fn run_client(config: ClientConfig) -> Result<()> {
	let payload = vec![0_u8; config.bench.payload_len];

	println!(
		"Config: messages={}, payload={} bytes, warmup_rounds={}, rounds={}.",
		config.bench.msg_count,
		config.bench.payload_len,
		config.bench.warmup_rounds,
		config.bench.rounds
	);

	match config.implementation {
		Implementation::Fastwebsockets =>
			fastwebsockets::run_client(config.addr, &config.bench, config.mode, &payload).await,
		Implementation::SockudoWs =>
			sockudo_ws::run_client(config.addr, &config.bench, config.mode, &payload).await,
		Implementation::Soketto =>
			soketto::run_client(config.addr, &config.bench, config.mode, &payload).await,
		Implementation::TokioTungstenite =>
			tokio_tungstenite::run_client(config.addr, &config.bench, config.mode, &payload).await,
		Implementation::TokioWebsockets =>
			tokio_websockets::run_client(config.addr, &config.bench, config.mode, &payload).await,
		Implementation::WsTool =>
			ws_tool::run_client(config.addr, &config.bench, config.mode, &payload).await,
	}
}

fn spawn_server(implementation: Implementation) -> Result<Child> {
	let exe = std::env::current_exe().wrap_err("Failed to resolve current executable.")?;

	Command::new(exe)
		.arg("server")
		.arg("--impl")
		.arg(implementation.as_str())
		.stdout(Stdio::piped())
		.stderr(Stdio::inherit())
		.spawn()
		.wrap_err("Failed to spawn server process.")
}

fn spawn_client(
	implementation: Implementation,
	mode: Mode,
	bench: &BenchConfig,
	port: u16,
) -> Result<Child> {
	let exe = std::env::current_exe().wrap_err("Failed to resolve current executable.")?;
	let addr = format!("127.0.0.1:{port}");

	let mut cmd = Command::new(exe);
	cmd.arg("client")
		.arg("--impl")
		.arg(implementation.as_str())
		.arg("--mode")
		.arg(match mode {
			Mode::Rtt => "rtt",
			Mode::Throughput => "throughput",
		})
		.arg("--addr")
		.arg(addr)
		.arg("--messages")
		.arg(bench.msg_count.to_string())
		.arg("--payload")
		.arg(bench.payload_len.to_string())
		.arg("--warmup")
		.arg(bench.warmup_rounds.to_string())
		.arg("--rounds")
		.arg(bench.rounds.to_string())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit());

	cmd.spawn().wrap_err("Failed to spawn client process.")
}

fn wait_for_ready(child: &mut Child) -> Result<u16> {
	let stdout = child.stdout.take().ok_or_else(|| eyre!("Server stdout was not captured."))?;
	let (tx, rx) = mpsc::channel::<u16>();

	std::thread::spawn(move || {
		let reader = io::BufReader::new(stdout);
		for line in reader.lines().map_while(Result::ok) {
			if let Some(port) = parse_ready_line(&line) {
				let _ = tx.send(port);
				break;
			}
		}
	});

	rx.recv_timeout(READY_TIMEOUT).map_err(|_| eyre!("Timed out waiting for server readiness."))
}

fn parse_ready_line(line: &str) -> Option<u16> {
	let prefix = "Ready on port ";
	let suffix = '.';
	if let Some(rest) = line.strip_prefix(prefix) {
		let rest = rest.strip_suffix(suffix)?;
		return rest.trim().parse::<u16>().ok();
	}
	None
}

fn parse_args(args: &[String]) -> Result<ConfigUnion> {
	if args.is_empty() {
		return Ok(ConfigUnion::Driver(parse_driver_args(&mut std::iter::empty::<String>())?));
	}

	match args[0].as_str() {
		"server" => Ok(ConfigUnion::Server(parse_server_args(&mut args[1..].iter().cloned())?)),
		"client" => Ok(ConfigUnion::Client(parse_client_args(&mut args[1..].iter().cloned())?)),
		"driver" => Ok(ConfigUnion::Driver(parse_driver_args(&mut args[1..].iter().cloned())?)),
		value if value.starts_with("--") =>
			Ok(ConfigUnion::Driver(parse_driver_args(&mut args.iter().cloned())?)),
		value => Err(eyre!("Unknown role '{value}'. Use driver, server, or client.")),
	}
}

enum ConfigUnion {
	Driver(DriverConfig),
	Server(ServerConfig),
	Client(ClientConfig),
}

fn parse_driver_args<I>(iter: &mut I) -> Result<DriverConfig>
where
	I: Iterator<Item = String>,
{
	let mut impls: Option<Vec<Implementation>> = None;
	let mut bench = default_bench_config();

	while let Some(arg) = iter.next() {
		match arg.as_str() {
			"--impl" => {
				let value = next_value(iter, "--impl")?;
				impls = Some(parse_impl_list(&value)?);
			},
			"--messages" => bench.msg_count = parse_u32(next_value(iter, "--messages")?)?,
			"--payload" => bench.payload_len = parse_usize(next_value(iter, "--payload")?)?,
			"--warmup" => bench.warmup_rounds = parse_u32(next_value(iter, "--warmup")?)?,
			"--rounds" => bench.rounds = parse_u32(next_value(iter, "--rounds")?)?,
			"--help" => {
				print_usage();
				std::process::exit(0);
			},
			_ => {
				return Err(eyre!(
					"Unknown argument '{arg}'. Use --impl, --messages, --payload, --warmup, --rounds, or --help."
				));
			},
		}
	}

	validate_bench_config(&bench)?;

	Ok(DriverConfig { impls: impls.unwrap_or_else(Implementation::all), bench })
}

fn parse_server_args<I>(iter: &mut I) -> Result<ServerConfig>
where
	I: Iterator<Item = String>,
{
	let mut implementation = None;

	while let Some(arg) = iter.next() {
		match arg.as_str() {
			"--impl" => implementation = Some(Implementation::parse(&next_value(iter, "--impl")?)?),
			"--help" => {
				print_usage();
				std::process::exit(0);
			},
			_ => {
				return Err(eyre!("Unknown argument '{arg}'. Use --impl or --help."));
			},
		}
	}

	Ok(ServerConfig {
		implementation: implementation.ok_or_else(|| eyre!("Missing required --impl flag."))?,
	})
}

fn parse_client_args<I>(iter: &mut I) -> Result<ClientConfig>
where
	I: Iterator<Item = String>,
{
	let mut implementation = None;
	let mut addr = None;
	let mut mode = None;
	let mut bench = default_bench_config();

	while let Some(arg) = iter.next() {
		match arg.as_str() {
			"--impl" => implementation = Some(Implementation::parse(&next_value(iter, "--impl")?)?),
			"--addr" => addr = Some(parse_addr(&next_value(iter, "--addr")?)?),
			"--mode" => mode = Some(parse_mode(&next_value(iter, "--mode")?)?),
			"--messages" => bench.msg_count = parse_u32(next_value(iter, "--messages")?)?,
			"--payload" => bench.payload_len = parse_usize(next_value(iter, "--payload")?)?,
			"--warmup" => bench.warmup_rounds = parse_u32(next_value(iter, "--warmup")?)?,
			"--rounds" => bench.rounds = parse_u32(next_value(iter, "--rounds")?)?,
			"--help" => {
				print_usage();
				std::process::exit(0);
			},
			_ => {
				return Err(eyre!(
					"Unknown argument '{arg}'. Use --impl, --addr, --mode, --messages, --payload, --warmup, --rounds, or --help."
				));
			},
		}
	}

	validate_bench_config(&bench)?;

	Ok(ClientConfig {
		implementation: implementation.ok_or_else(|| eyre!("Missing required --impl flag."))?,
		bench,
		mode: mode.ok_or_else(|| eyre!("Missing required --mode flag."))?,
		addr: addr.ok_or_else(|| eyre!("Missing required --addr flag."))?,
	})
}

fn default_bench_config() -> BenchConfig {
	BenchConfig {
		msg_count: DEFAULT_MESSAGE_COUNT,
		payload_len: DEFAULT_PAYLOAD_SIZE,
		warmup_rounds: DEFAULT_WARMUP_ROUNDS,
		rounds: DEFAULT_ROUNDS,
	}
}

fn validate_bench_config(config: &BenchConfig) -> Result<()> {
	if config.msg_count == 0 {
		return Err(eyre!("Message count must be greater than zero."));
	}
	if config.payload_len == 0 {
		return Err(eyre!("Payload length must be greater than zero."));
	}
	if config.rounds == 0 {
		return Err(eyre!("Rounds must be greater than zero."));
	}
	Ok(())
}

fn parse_impl_list(value: &str) -> Result<Vec<Implementation>> {
	value.split(',').map(|item| Implementation::parse(item.trim())).collect()
}

fn parse_mode(value: &str) -> Result<Mode> {
	match value {
		"rtt" => Ok(Mode::Rtt),
		"throughput" => Ok(Mode::Throughput),
		_ => Err(eyre!("Unknown mode '{value}'. Use rtt or throughput.")),
	}
}

fn parse_addr(value: &str) -> Result<std::net::SocketAddr> {
	value.parse().wrap_err("Address must be in host:port format.")
}

fn parse_u32(value: String) -> Result<u32> {
	value.parse().wrap_err("Expected an unsigned integer.")
}

fn parse_usize(value: String) -> Result<usize> {
	value.parse().wrap_err("Expected an unsigned integer.")
}

fn next_value<I>(iter: &mut I, flag: &str) -> Result<String>
where
	I: Iterator<Item = String>,
{
	iter.next().ok_or_else(|| eyre!("Missing value for {flag} flag."))
}

fn print_usage() {
	println!(
		"Usage:\n  websocket-benchmark [driver] [--impl <list>] [--messages <n>] [--payload <bytes>] [--warmup <n>] [--rounds <n>]\n  websocket-benchmark server --impl <name>\n  websocket-benchmark client --impl <name> --mode <rtt|throughput> --addr <host:port> [--messages <n>] [--payload <bytes>] [--warmup <n>] [--rounds <n>]\n\nExamples:\n  websocket-benchmark\n  websocket-benchmark --impl fastwebsockets,sockudo-ws,soketto\n  websocket-benchmark client --impl soketto --mode rtt --addr 127.0.0.1:9001"
	);
}

pub(crate) fn set_nodelay(stream: &tokio::net::TcpStream) -> Result<()> {
	stream.set_nodelay(true).wrap_err("Failed to enable TCP_NODELAY.")
}
