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

/// Format EVM block number (big-endian bytes from proto) as decimal string.
/// Empty yields "0". Used when filter blocks need block number.
#[allow(dead_code)]
pub fn format_block_number_be(bytes: &[u8]) -> String {
    format!("{}", bytes.iter().fold(0u64, |n, &b| n << 8 | b as u64))
}

const HEX_CHARS: [u8; 16] = *b"0123456789abcdef";

fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(2 + bytes.len() * 2);
    s.push('0');
    s.push('x');
    for &b in bytes {
        s.push(HEX_CHARS[(b >> 4) as usize] as char);
        s.push(HEX_CHARS[(b & 0xf) as usize] as char);
    }
    s
}

/// Format raw bytes for display using the chain's convention.
/// O(n) in the length of `bytes`; one allocation for the result.
pub fn format_bytes(bytes: &[u8], encoding: ChainEncoding) -> String {
    match encoding {
        ChainEncoding::Solana => bs58::encode(bytes).into_string(),
        ChainEncoding::Evm | ChainEncoding::Tron => encode_hex(bytes),
    }
}

/// Write formatted bytes into `buf` and return `buf` as &str. Reusing `buf` avoids per-call allocation for EVM hex.
pub fn format_bytes_into<'a>(bytes: &[u8], encoding: ChainEncoding, buf: &'a mut String) -> &'a str {
    buf.clear();
    match encoding {
        ChainEncoding::Solana => {
            buf.push_str(&bs58::encode(bytes).into_string());
        }
        ChainEncoding::Evm | ChainEncoding::Tron => {
            buf.reserve(2 + bytes.len() * 2);
            buf.push('0');
            buf.push('x');
            for &b in bytes {
                buf.push(HEX_CHARS[(b >> 4) as usize] as char);
                buf.push(HEX_CHARS[(b & 0xf) as usize] as char);
            }
        }
    }
    buf.as_str()
}
