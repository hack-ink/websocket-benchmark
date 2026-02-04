<div align="center">

# websocket-benchmark

### Rust websocket benchmark.

[![License](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Rust](https://github.com/hack-ink/websocket-benchmark/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/hack-ink/websocket-benchmark/actions/workflows/rust.yml)
[![GitHub tag (latest by date)](https://img.shields.io/github/v/tag/hack-ink/websocket-benchmark)](https://github.com/hack-ink/websocket-benchmark/tags)
[![GitHub last commit](https://img.shields.io/github/last-commit/hack-ink/websocket-benchmark?color=red&style=plastic)](https://github.com/hack-ink/websocket-benchmark)

</div>

### Usage

```sh
cargo run --release
```

Optional flags:

```sh
cargo run --release -- --impl fastwebsockets,sockudo-ws,soketto --messages 100000 --payload 4096 --warmup 1 --rounds 5
```

Supported implementations: fastwebsockets, sockudo-ws, soketto, tokio-tungstenite, tokio-websockets, ws-tool.

Advanced roles:

```sh
websocket-benchmark server --impl soketto
websocket-benchmark client --impl soketto --mode rtt --addr 127.0.0.1:9001
```

### Example Output

Values will vary by hardware and settings.

- Apple Silicon M4 MAX 64GB

    ```
    Benchmarking soketto.
    Phase: RTT.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    RTT result (us): median=30.31, p90=31.10, p99=32.44, mean=30.62, stdev=0.52.
    Phase: Throughput.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    Throughput result (MiB/s, tx+rx): median=750.12, p90=760.88, p99=772.54, mean=752.30, stdev=8.10.

    Benchmarking tokio-tungstenite.
    Phase: RTT.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    RTT result (us): median=19.99, p90=20.45, p99=21.03, mean=20.12, stdev=0.31.
    Phase: Throughput.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    Throughput result (MiB/s, tx+rx): median=820.14, p90=832.02, p99=845.91, mean=823.55, stdev=9.44.
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
