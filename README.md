# evm-solana-kafka-examples

Rust Kafka consumer for Bitquery blockchain streams (Solana and EVM: Base, Ethereum, BSC, Tron). Consumes protobuf messages and runs example filters.

## Prerequisites

- [Rust](https://rustup.rs/) (e.g. `rustup`)
- A `config.toml` file (an example is included in the repo)

## Configuration

Edit [config.toml](config.toml):

- Set `chain` to one of: `"solana"`, `"base"`, `"ethereum"`, `"bsc"`, `"tron"`.
- Fill the matching section (`[solana]`, `[base]`, etc.) with your Bitquery `username`, `password`, and `topic`.

## Build and run

```bash
cargo build
cargo run
```

Run from the project root so that `config.toml` is in the current working directory (the app loads it via the `config` crate).

To add or modify event filters, edit [src/filters.rs](src/filters.rs).

## Supported chains

- **Solana** (Solana proto schema)
- **EVM:** Base, Ethereum, BSC, Tron (shared EVM proto schema)
