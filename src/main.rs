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

mod soketto;
mod tokio_tungstenite;
mod tokio_websockets;
mod ws_tool;

#[tokio::main]
async fn main() {
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
}

#[test]
fn placeholder() {}
