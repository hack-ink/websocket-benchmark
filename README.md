<div align="center">

# websocket-benchmark

### Rust websocket benchmark.

[![License](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Language Checks](https://github.com/hack-ink/websocket-benchmark/actions/workflows/language.yml/badge.svg?branch=main)](https://github.com/hack-ink/websocket-benchmark/actions/workflows/language.yml)
[![GitHub tag (latest by date)](https://img.shields.io/github/v/tag/hack-ink/websocket-benchmark)](https://github.com/hack-ink/websocket-benchmark/tags)
[![GitHub last commit](https://img.shields.io/github/last-commit/hack-ink/websocket-benchmark?color=red&style=plastic)](https://github.com/hack-ink/websocket-benchmark)
[![GitHub code lines](https://tokei.rs/b1/github/hack-ink/websocket-benchmark)](https://github.com/hack-ink/websocket-benchmark)

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
    RTT result (us): median=33.86, p90=33.95, p99=33.99, mean=33.66, stdev=0.36.
    Phase: Throughput.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    Throughput result (MiB/s, tx+rx): median=973.49, p90=996.31, p99=1000.86, mean=980.12, stdev=13.04.

    Benchmarking tokio-tungstenite.
    Phase: RTT.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    RTT result (us): median=28.23, p90=29.28, p99=29.61, mean=27.70, stdev=1.67.
    Phase: Throughput.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    Throughput result (MiB/s, tx+rx): median=2224.08, p90=2443.26, p99=2496.92, mean=2281.33, stdev=134.83.

    Benchmarking tokio-websockets.
    Phase: RTT.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    RTT result (us): median=24.81, p90=25.35, p99=25.48, mean=24.97, stdev=0.31.
    Phase: Throughput.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    Throughput result (MiB/s, tx+rx): median=2453.13, p90=2531.10, p99=2552.74, mean=2400.84, stdev=156.63.

    Benchmarking ws-tool.
    Phase: RTT.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    RTT result (us): median=24.94, p90=25.73, p99=26.15, mean=25.05, stdev=0.61.
    Phase: Throughput.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    Throughput result (MiB/s, tx+rx): median=3506.21, p90=3575.85, p99=3606.07, mean=3204.69, stdev=477.82.

    Benchmarking fastwebsockets.
    Phase: RTT.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    RTT result (us): median=26.46, p90=26.55, p99=26.58, mean=25.95, stdev=0.98.
    Phase: Throughput.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    Throughput result (MiB/s, tx+rx): median=2606.39, p90=2639.46, p99=2639.57, mean=2596.05, stdev=42.31.

    Benchmarking sockudo-ws.
    Phase: RTT.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    RTT result (us): median=27.88, p90=28.23, p99=28.32, mean=27.96, stdev=0.22.
    Phase: Throughput.
    Config: messages=100000, payload=4096 bytes, warmup_rounds=1, rounds=5.
    Throughput result (MiB/s, tx+rx): median=2274.14, p90=2461.63, p99=2529.10, mean=2310.82, stdev=134.29.
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
