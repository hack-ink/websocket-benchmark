//! Rust websocket benchmark.

#![deny(clippy::all, missing_docs, unused_crate_dependencies)]

mod prelude {
	pub use std::time::{Duration, Instant};

	pub use futures::prelude::*;
	pub use tokio::{net::TcpListener, sync::oneshot};

	pub const ADDR: &str = "0.0.0.0:9001";
	pub const MSG_COUNT: u32 = 100_000;
	pub const PAYLOAD: &[u8] = &[0; 4_096];
}

mod fastwebsockets;
mod sockudo_ws;
mod soketto;
mod tokio_tungstenite;
mod tokio_websockets;
mod ws_tool;
use tokio::runtime::Builder;

fn main() {
	let runtime = Builder::new_multi_thread()
		.enable_all()
		.event_interval(1)
		.global_queue_interval(1)
		.thread_stack_size(3 * 1024 * 1024)
		.build()
		.expect("Failed to build Tokio runtime");

	runtime.block_on(async {
		color_eyre::install().unwrap();

		println!("Benchmarking soketto...");
		soketto::bench().await;
		println!();

		println!("Benchmarking tokio-tungstenite...");
		tokio_tungstenite::bench().await;
		println!();

		println!("Benchmarking tokio-websockets...");
		tokio_websockets::bench().await;
		println!();

		println!("Benchmarking ws-tool...");
		ws_tool::bench().await;
		println!();

		println!("Benchmarking fastwebsockets...");
		fastwebsockets::bench().await;
		println!();

		println!("Benchmarking sockudo-ws...");
		sockudo_ws::bench().await;
		println!();
	});
}

#[test]
fn placeholder() {}
