# DTChat - Application for Delay Tolerant Communications

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE) [![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)


### Quick test


```
# First terminal
CONFIG_PATH=./db/conf.yaml ENGINE_RECEIVE_DELAY_MS=3000 PEER_UUID=1 cargo run --features=with_delay
# Second terminal
CONFIG_PATH=./db/conf.yaml ENGINE_RECEIVE_DELAY_MS=7000 PEER_UUID=2 cargo run --features=with_delay
# Third terminal
CONFIG_PATH=./db/conf.yaml ENGINE_RECEIVE_DELAY_MS=3000 PEER_UUID=3 cargo run --features=with_delay
```

(The with_delay features allows the implementation of articial delays to match the CP delays for internet tests)
