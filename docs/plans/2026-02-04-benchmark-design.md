# Benchmark Design

## Goal

Deliver a fair, consistent, and repeatable WebSocket benchmark across libraries, with minimal extra dependencies and clear output for RTT and throughput.

## Architecture

The binary is split into three roles: driver, server, and client. The driver orchestrates runs for each library and phase by spawning a dedicated server process and then a client process. The server binds to `127.0.0.1:0` and reports readiness with the selected port. The client runs either RTT or throughput on a single connection with a warmup phase and multiple measured rounds. The server accepts a single connection and exits after a clean close.

## Consistency Rules

All libraries use the same message count, payload size, warmup rounds, and measurement rounds. TCP_NODELAY is enabled on both the client and server sockets before the WebSocket handshake when the library allows access to the underlying `TcpStream`. The throughput test measures full duplex payload bytes (tx + rx) to reflect the echo workload. Each phase reports median, p90, p99, mean, and standard deviation.

## Error Handling and Output

Errors are propagated with clear English messages and nonzero exit codes. Output is standardized to include the benchmark phase, configuration, and results, with units explicitly noted. The driver fails fast if the server readiness line is not observed within a short timeout.
