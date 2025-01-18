<div align="center">

# websocket-benchmark
### Rust websocket benchmark.

[![License](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Checks](https://github.com/hack-ink/websocket-benchmark/actions/workflows/checks.yml/badge.svg?branch=main)](https://github.com/hack-ink/websocket-benchmark/actions/workflows/checks.yml)
[![GitHub tag (latest by date)](https://img.shields.io/github/v/tag/hack-ink/websocket-benchmark)](https://github.com/hack-ink/websocket-benchmark/tags)
[![GitHub last commit](https://img.shields.io/github/last-commit/hack-ink/websocket-benchmark?color=red&style=plastic)](https://github.com/hack-ink/websocket-benchmark)
</div>


### Usage
```sh
cargo run --release
```

### Result
- Apple Silicon M4 MAX 64GB
  ```
  Benchmarking soketto...
  server: listening on 0.0.0.0:9001
  server: new connection from 127.0.0.1:53127
  server: WS handshake successful
  client: WS connected
  client: sent 100000 messages, average round trip time 30.31µs
  server: received close signal

  Benchmarking tokio-tungstenite...
  server: listening on 0.0.0.0:9001
  server: new connection from 127.0.0.1:53150
  server: WS handshake successful
  client: WS connected
  client: sent 100000 messages, average round trip time 19.99µs
  server: received close signal

  Benchmarking tokio-websockets...
  server: listening on 0.0.0.0:9001
  server: new connection from 127.0.0.1:53158
  server: WS handshake successful
  client: WS connected
  client: sent 100000 messages, average round trip time 23.84µs
  server: received close signal

  Benchmarking ws-tool...
  server: listening on 0.0.0.0:9001
  server: new connection from 127.0.0.1:53168
  server: WS handshake successful
  client: WS connected
  client: sent 100000 messages, average round trip time 19.84µs
  server: received close signal
  ```


## Support Me
If you find this project helpful and would like to support its development, you can buy me a coffee!

Your support is greatly appreciated and motivates me to keep improving this project.

- **Fiat**
  - [Ko-fi](https://ko-fi.com/hack_ink)
  - [爱发电](https://afdian.com/a/hack_ink)
- **Crypto**
  - **Bitcoin**
    - `bc1pedlrf67ss52md29qqkzr2avma6ghyrt4jx9ecp9457qsl75x247sqcp43c`
  - **Ethereum**
    - `0x3e25247CfF03F99a7D83b28F207112234feE73a6`
  - **Polkadot**
    - `156HGo9setPcU2qhFMVWLkcmtCEGySLwNqa3DaEiYSWtte4Y`

Thank you for your support!


## Appreciation
We would like to extend our heartfelt gratitude to the following projects and contributors:
- https://github.com/nurmohammed840/web-socket-benchmark


<div align="right">

### License
<sup>Licensed under [GPL-3.0](LICENSE).</sup>
</div>
