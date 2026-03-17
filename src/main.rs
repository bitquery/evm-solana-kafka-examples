mod encoding;
mod filters;
mod protos;

use rdkafka::config::ClientConfig;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::Message;
use rdkafka::error::KafkaResult;
use tokio::time::{timeout, Duration};
use prost::Message as ProstMessage;
use crate::encoding::ChainEncoding;
use crate::protos::solana::{
    BlockMessage as SolanaBlockMessage,
    DexParsedBlockMessage as SolanaDexParsedBlockMessage,
    ParsedIdlBlockMessage as SolanaParsedIdlBlockMessage,
    TokenBlockMessage as SolanaTokenBlockMessage,
};
use crate::protos::evm::evm_messages::{
    DexBlockMessage as EvmDexBlockMessage,
    ParsedAbiBlockMessage as EvmParsedAbiBlockMessage,
    TokenBlockMessage as EvmTokenBlockMessage,
};
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
    #[serde(default)]
    solana: Option<AuthConfig>,
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
        "solana" => settings.solana.as_ref().ok_or_else(|| format!("config: [solana] not set for chain={}", settings.chain)),
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
                            if let Ok(block) = EvmParsedAbiBlockMessage::decode(payload) {
                                filters::evm_parsed_abi(&block, encoding, &settings.chain);
                            } else if let Ok(block) = EvmDexBlockMessage::decode(payload) {
                                filters::evm_dex(&block, encoding);
                            } else if let Ok(block) = EvmTokenBlockMessage::decode(payload) {
                                filters::evm_token(&block, encoding);
                            } else {
                                eprintln!("Failed to decode EVM message (tried ParsedAbiBlockMessage, DexBlockMessage, TokenBlockMessage)");
                            }
                        } else {
                            // Solana: try all message types (dex, token, parsed_idl, block)
                            if let Ok(block) = SolanaDexParsedBlockMessage::decode(payload) {
                                filters::solana_dex(&block, encoding);
                            } else if let Ok(block) = SolanaTokenBlockMessage::decode(payload) {
                                filters::solana_token(&block, encoding);
                            } else if let Ok(block) = SolanaParsedIdlBlockMessage::decode(payload) {
                                filters::solana_parsed_idl(&block, encoding);
                            } else if let Ok(block) = SolanaBlockMessage::decode(payload) {
                                filters::solana_block(&block, encoding);
                            } else {
                                eprintln!("Failed to decode Solana message (tried DexParsedBlockMessage, TokenBlockMessage, ParsedIdlBlockMessage, BlockMessage)");
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
