//! Event filtering and print logic for decoded block messages.
//! Main.rs only does decode dispatch; this module holds all condition checks and output.

use std::sync::OnceLock;
use chrono::Local;
use crate::encoding::{format_bytes, format_bytes_into, ChainEncoding};
use crate::protos::evm::evm_messages::{
    DexBlockMessage as EvmDexBlockMessage,
    ParsedAbiBlockMessage as EvmParsedAbiBlockMessage,
    TokenBlockMessage as EvmTokenBlockMessage,
};

/// Contract address for meme token creation (Transaction.To). Byte comparison avoids format_bytes allocation.
//0x5c952063c7fc8610ffdb798152d69f0b9550762b (bsc)
const MEME_TOKEN_CREATE_TO_BYTES: [u8; 20] = [
    0x5c, 0x95, 0x20, 0x63, 0xc7, 0xfc, 0x86, 0x10, 0xff, 0xdb, 0x79, 0x81, 0x52, 0xd6, 0x9f,
    0x0b, 0x95, 0x50, 0x76, 0x2b,
];

use crate::protos::solana::{
    BlockMessage as SolanaBlockMessage,
    DexParsedBlockMessage as SolanaDexParsedBlockMessage,
    ParsedIdlBlockMessage as SolanaParsedIdlBlockMessage,
    TokenBlockMessage as SolanaTokenBlockMessage,
};

// Solana Launchpad program addresses (base58) for Solana token-creation filter.
const SOLANA_PGM_6EF8: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
const SOLANA_PGM_LAN: &str = "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj";
const SOLANA_PGM_DBCIJ: &str = "dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN";
const SOLANA_PGM_HEAVEN: &str = "HEAVENoP2qxoeuF8Dj2oT1GHEnu49U5mJYkdeC8BAX2o";
const SOLANA_PGM_MOON: &str = "MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG";
const SOLANA_ACCT_BAGSB: &str = "BAGSB9TpGrZxQbEsrEznv5jXXdwyP6AXerN8aVRiAmcv";

// Pre-decoded base58 program/account bytes (decoded once per constant).
fn solana_pgm_6ef8_bytes() -> &'static [u8] {
    static C: OnceLock<Vec<u8>> = OnceLock::new();
    C.get_or_init(|| bs58::decode(SOLANA_PGM_6EF8).into_vec().unwrap()).as_slice()
}
fn solana_pgm_lan_bytes() -> &'static [u8] {
    static C: OnceLock<Vec<u8>> = OnceLock::new();
    C.get_or_init(|| bs58::decode(SOLANA_PGM_LAN).into_vec().unwrap()).as_slice()
}
fn solana_pgm_dbcij_bytes() -> &'static [u8] {
    static C: OnceLock<Vec<u8>> = OnceLock::new();
    C.get_or_init(|| bs58::decode(SOLANA_PGM_DBCIJ).into_vec().unwrap()).as_slice()
}
fn solana_pgm_heaven_bytes() -> &'static [u8] {
    static C: OnceLock<Vec<u8>> = OnceLock::new();
    C.get_or_init(|| bs58::decode(SOLANA_PGM_HEAVEN).into_vec().unwrap()).as_slice()
}
fn solana_pgm_moon_bytes() -> &'static [u8] {
    static C: OnceLock<Vec<u8>> = OnceLock::new();
    C.get_or_init(|| bs58::decode(SOLANA_PGM_MOON).into_vec().unwrap()).as_slice()
}
fn solana_acct_bagsb_bytes() -> &'static [u8] {
    static C: OnceLock<Vec<u8>> = OnceLock::new();
    C.get_or_init(|| bs58::decode(SOLANA_ACCT_BAGSB).into_vec().unwrap()).as_slice()
}

/// Parsed ABI topic: token creation = Call.Create == true; pair created = log.Parsed.Signature.Name == "PairCreated"
/// For BSC, meme migration is labeled "BSC fourmeme migration" (same condition: PairCreated + Transaction.To).
/// Single pass over calls/logs per tx to set create_any, pair_created, has_token_create_log.
pub fn evm_parsed_abi(block: &EvmParsedAbiBlockMessage, encoding: ChainEncoding, chain: &str) {
    let mut buf = String::new();
    for (_tx_idx, tx) in block.transactions.iter().enumerate() {
        let to_matches = tx
            .transaction_header
            .as_ref()
            .map(|th| th.to.as_ref() == MEME_TOKEN_CREATE_TO_BYTES)
            .unwrap_or(false);

        let mut create_any = false;
        let mut pair_created = false;
        let mut has_token_create_log = false;
        for call in &tx.calls {
            if call.header.as_ref().map(|h| h.create).unwrap_or(false) {
                create_any = true;
            }
            for log in &call.logs {
                if let Some(name) = log
                    .parsed
                    .as_ref()
                    .and_then(|p| p.signature.as_ref())
                    .map(|s| s.name.as_str())
                {
                    if name == "PairCreated" {
                        pair_created = true;
                    } else if name == "TokenCreate" {
                        has_token_create_log = true;
                    }
                }
            }
        }

        let th = match &tx.transaction_header {
            Some(h) => h,
            None => continue,
        };

        if create_any {
            println!("--- EVM Token creation (Call.Create = true) ---");
            println!("  TxHash: {}", format_bytes_into(&th.hash, encoding, &mut buf));
        }
        if pair_created {
            println!("--- EVM PairCreated ---");
            println!("  TxHash: {}", format_bytes_into(&th.hash, encoding, &mut buf));
        }
        if to_matches && has_token_create_log {
            println!("--- EVM Meme token creation (TokenCreate) ---");
            println!("  TxHash: {}", format_bytes_into(&th.hash, encoding, &mut buf));
        }
        if to_matches && pair_created {
            if chain.eq_ignore_ascii_case("bsc") {
                println!("--- BSC fourmeme migration ---");
            } else {
                println!("--- EVM Meme token migration (PairCreated) ---");
            }
            println!("  TxHash: {}", format_bytes_into(&th.hash, encoding, &mut buf));
        }
    }
}

/// EVM Dex block: optional filter blocks (token creation / migration).
pub fn evm_dex(block: &EvmDexBlockMessage, _encoding: ChainEncoding) {
    #[allow(unused_variables)]
    for trade in &block.trades {
        // Print details only when the trade matches the criteria below.
        // ----- Set of conditions A: uncomment for token creation on xyz -----
        // if evm: e.g. trade.dex.protocol_name == "xyz" or first-seen Currency.SmartContract
        // if let Some(ref dex) = trade.dex {
        //     if dex.protocol_name == "xyz" { ... }
        // }
        // if /* condition A */ {
        //     println!("--- EVM Trade (token creation) ---");
        //     println!("  TxIndex: {}", trade.transaction_index);
        //     println!("  Success: {}", trade.success);
        // }
    }
}

/// EVM Token block (e.g. base.tokens.proto): optional filter blocks.
pub fn evm_token(block: &EvmTokenBlockMessage, _encoding: ChainEncoding) {
    #[allow(unused_variables)]
    for transfer in &block.transfers {
        // Print details only when the transfer matches the criteria below.
        // ----- Set of conditions A-----
        
    }
}

/// Solana Dex block: optional filter blocks. Print only tx/signature when matched.
pub fn solana_dex(block: &SolanaDexParsedBlockMessage, _encoding: ChainEncoding) {
    #[allow(unused_variables)]
    for dex_tx in &block.transactions {
        // Print details only when the transaction matches the criteria below.
        // ----- Set of conditions A:-----
       

    
    }
}

/// Solana Token block: placeholder filter blocks. Print only tx/signature when matched.
pub fn solana_token(block: &SolanaTokenBlockMessage, _encoding: ChainEncoding) {
    #[allow(unused_variables)]
    for _tx in &block.transactions {
        // ----- Set of conditions A:  -----
        // ----- Set of conditions B: -----
    }
}

/// Returns true if this instruction is a PumpFun migrate (Program 6EF8, logs include "Migrate").
fn is_pumpfun_migrate_instruction(inst: &crate::protos::solana::ParsedIdlInstruction) -> bool {
    let program = match &inst.program {
        Some(p) => p,
        None => return false,
    };
    let addr: &[u8] = program.address.as_ref();
    if addr != solana_pgm_6ef8_bytes() {
        return false;
    }
    inst.logs.iter().any(|s| s.contains("Migrate"))
}

/// Returns true if this instruction matches any launchpad token-creation condition (for Solana).
fn is_launchpad_token_creation_instruction(
    inst: &crate::protos::solana::ParsedIdlInstruction,
    encoding: ChainEncoding,
) -> bool {
    let program = match &inst.program {
        Some(p) => p,
        None => return false,
    };
    let addr: &[u8] = program.address.as_ref();
    let method = program.method.as_str();
    // 6EF8: create | create_v2
    if addr == solana_pgm_6ef8_bytes() && (method == "create" || method == "create_v2") {
        return true;
    }
    // Lan: initialize_v2
    if addr == solana_pgm_lan_bytes() && method == "initialize_v2" {
        return true;
    }
    // dbcij3: initialize_virtual_pool_with_spl_token (plain or with account conditions)
    if addr == solana_pgm_dbcij_bytes() && method == "initialize_virtual_pool_with_spl_token" {
        let bagsb = solana_acct_bagsb_bytes();
        let has_bagsb = inst.accounts.iter().any(|a| {
            let a_addr: &[u8] = a.address.as_ref();
            a_addr == bagsb
        });
        let has_jups = inst.accounts.iter().any(|a| {
            let s = format_bytes(&a.address, encoding);
            s.ends_with("jups")
        });
        if has_bagsb || has_jups {
            return true;
        }
        // no account filter: still counts as launchpad create
        return true;
    }
    // HEAVEN: create_standard_liquidity_pool
    if addr == solana_pgm_heaven_bytes() && method == "create_standard_liquidity_pool" {
        return true;
    }
    // Moon: tokenMint
    if addr == solana_pgm_moon_bytes() && method == "tokenMint" {
        return true;
    }
    false
}

/// Solana ParsedIdl block: launchpad token-creation filter. Print only tx hash when matched.
/// Single pass over parsed_idl_instructions to set both launchpad and pumpfun migrate flags.
pub fn solana_parsed_idl(block: &SolanaParsedIdlBlockMessage, encoding: ChainEncoding) {
    let mut buf = String::new();
    for tx in &block.transactions {
        let success = tx
            .status
            .as_ref()
            .map(|s| s.success)
            .unwrap_or(false);
        if !success {
            continue;
        }
        let mut is_launchpad = false;
        let mut is_pumpfun_migrate = false;
        for inst in &tx.parsed_idl_instructions {
            if is_launchpad_token_creation_instruction(inst, encoding) {
                is_launchpad = true;
            }
            if is_pumpfun_migrate_instruction(inst) {
                is_pumpfun_migrate = true;
            }
        }
        if is_launchpad {
            println!("--- Solana launchpad token creation ---");
            println!("  TxHash: {}", format_bytes_into(&tx.signature, encoding, &mut buf));
        }
        if is_pumpfun_migrate {
            println!("--- Solana migration to pumpswap from pumpfun ---");
            println!("  TxHash: {}", format_bytes_into(&tx.signature, encoding, &mut buf));
        }
    }
}

/// Solana raw BlockMessage: block with transactions and rewards. Print only tx/signature when matched.
pub fn solana_block(_block: &SolanaBlockMessage, _encoding: ChainEncoding) {
    // BlockMessage: block with transactions and rewards
}
