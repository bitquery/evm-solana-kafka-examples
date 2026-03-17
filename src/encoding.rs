//! Chain-specific encoding for bytes (addresses, hashes, signatures).
//! Solana: base58. EVM / Tron: hex with 0x prefix.

use bs58;

#[derive(Debug, Clone, Copy)]
pub enum ChainEncoding {
    /// Solana: addresses and signatures in base58.
    Solana,
    /// EVM chains (base, ethereum, bsc): hex with 0x prefix.
    Evm,
    /// Tron: hex with 0x prefix (same as EVM).
    Tron,
}

impl ChainEncoding {
    pub fn from_chain_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "solana" => Some(ChainEncoding::Solana),
            "base" | "ethereum" | "bsc" => Some(ChainEncoding::Evm),
            "tron" => Some(ChainEncoding::Tron),
            _ => None,
        }
    }
}

/// Format raw bytes for display using the chain's convention.
/// O(n) in the length of `bytes`; one allocation for the result.
pub fn format_bytes(bytes: &[u8], encoding: ChainEncoding) -> String {
    match encoding {
        ChainEncoding::Solana => bs58::encode(bytes).into_string(),
        ChainEncoding::Evm | ChainEncoding::Tron => {
            let hex: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
            format!("0x{}", hex)
        }
    }
}
