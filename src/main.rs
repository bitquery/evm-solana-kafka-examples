mod encoding;
mod protos;

use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::error::KafkaResult;
use tokio::time::{timeout, Duration};
use prost::Message as ProstMessage;
use crate::encoding::{ChainEncoding, format_bytes};
use crate::protos::solana::DexParsedBlockMessage;
use crate::protos::evm::evm_messages::DexBlockMessage as EvmDexBlockMessage;
use config::Config;
use serde::Deserialize;

/// Shared auth + topic config for any chain (Solana or EVM: base/ethereum/bsc).
#[derive(Debug, Deserialize)]
struct AuthConfig {
    username: String,
    password: String,
    topic: String,
}

#[derive(Debug, Deserialize)]
struct Settings {
    /// Which chain to consume: "solana" | "base" | "ethereum" | "bsc" | "tron"
    chain: String,
    solana: AuthConfig,
    #[serde(default)]
    base: Option<AuthConfig>,
    #[serde(default)]
    ethereum: Option<AuthConfig>,
    #[serde(default)]
    bsc: Option<AuthConfig>,
    #[serde(default)]
    tron: Option<AuthConfig>,
}

fn get_auth_for_chain(settings: &Settings) -> Result<&AuthConfig, String> {
    match settings.chain.to_lowercase().as_str() {
        "solana" => Ok(&settings.solana),
        "base" => settings.base.as_ref().ok_or_else(|| format!("config: [base] not set for chain={}", settings.chain)),
        "ethereum" => settings.ethereum.as_ref().ok_or_else(|| format!("config: [ethereum] not set for chain={}", settings.chain)),
        "bsc" => settings.bsc.as_ref().ok_or_else(|| format!("config: [bsc] not set for chain={}", settings.chain)),
        "tron" => settings.tron.as_ref().ok_or_else(|| format!("config: [tron] not set for chain={}", settings.chain)),
        _ => Err(format!("unknown chain: {}", settings.chain)),
    }
}

fn is_evm_chain(chain: &str) -> bool {
    matches!(chain.to_lowercase().as_str(), "base" | "ethereum" | "bsc" | "tron")
}

#[tokio::main]
async fn main() -> KafkaResult<()> {
    env_logger::init();

    // Load config from config.toml
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .unwrap();

    let settings: Settings = settings.try_deserialize().unwrap();

    let auth = get_auth_for_chain(&settings)
        .expect("invalid config: set chain and the matching [chain] section with username, password, topic");
    let encoding = ChainEncoding::from_chain_name(&settings.chain)
        .expect("invalid config: unknown chain");
    let is_evm = is_evm_chain(&settings.chain);

    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "rpk0.bitquery.io:9092,rpk1.bitquery.io:9092,rpk2.bitquery.io:9092")
        .set("security.protocol", "SASL_PLAINTEXT")
        .set("ssl.endpoint.identification.algorithm", "none")
        .set("sasl.mechanisms", "SCRAM-SHA-512")
        .set("sasl.username", &auth.username)
        .set("sasl.password", &auth.password)
        .set("group.id", &format!("{}-group-{}", auth.username, uuid::Uuid::new_v4()))
        .set("fetch.message.max.bytes", "10485760")
        .create()?;

    let topics: Vec<&str> = vec![&auth.topic];
    consumer.subscribe(&topics)?;

    println!("Chain: {}, listening on topics: {:?}", settings.chain, topics);

    loop {
        match timeout(Duration::from_secs(10), consumer.recv()).await {
            Ok(msg_result) => match msg_result {
                Ok(msg) => {
                    if let Some(payload) = msg.payload() {
                        if is_evm {
                            match EvmDexBlockMessage::decode(payload) {
                                Ok(block) => {
                                    if let Some(header) = &block.header {
                                        println!("Block: number={:?}, hash={}", header.number, format_bytes(&header.hash, encoding));
                                    }
                                    for trade in &block.trades {
                                        println!("--- EVM Trade ---");
                                        println!("  TxIndex: {}", trade.transaction_index);
                                        println!("  Success: {}", trade.success);
                                    }
                                }
                                Err(e) => eprintln!("Failed to decode EVM DexBlockMessage: {}", e),
                            }
                        } else {
                            match DexParsedBlockMessage::decode(payload) {
                                Ok(parsed_block) => {
                                    if let Some(header) = &parsed_block.header {
                                        println!("Block: slot={:?}, timestamp={:?}",
                                            header.slot, header.timestamp);
                                    }
                                    for dex_tx in &parsed_block.transactions {
                                        println!("--- Transaction ---");
                                        println!("  Index: {}", dex_tx.index);
                                        println!("  Signature: {}", format_bytes(&dex_tx.signature, encoding));
                                        if let Some(status) = &dex_tx.status {
                                            println!("  Status: success={}", status.success);
                                        }
                                    }
                                }
                                Err(e) => eprintln!("Failed to decode DexParsedBlockMessage: {}", e),
                            }
                        }
                    }
                    consumer.commit_message(&msg, CommitMode::Async)?;
                }
                Err(e) => eprintln!("Error receiving message from Kafka: {}", e),
            },
            Err(_) => println!("No new messages within 10 seconds..."),
        }
    }
}
