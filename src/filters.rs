//! Event filtering and print logic for decoded block messages.
//! Main.rs only does decode dispatch; this module holds all condition checks and output.

use crate::encoding::{format_bytes, ChainEncoding};
use crate::protos::evm::evm_messages::{
    DexBlockMessage as EvmDexBlockMessage,
    ParsedAbiBlockMessage as EvmParsedAbiBlockMessage,
    TokenBlockMessage as EvmTokenBlockMessage,
};

/// Contract address for meme token creation (Transaction.To). Compare using format_bytes(&th.to, encoding).
const MEME_TOKEN_CREATE_TO: &str = "0x5c952063c7fc8610ffdb798152d69f0b9550762b";

use crate::protos::solana::{
    BlockMessage as SolanaBlockMessage,
    DexParsedBlockMessage as SolanaDexParsedBlockMessage,
    ParsedIdlBlockMessage as SolanaParsedIdlBlockMessage,
    TokenBlockMessage as SolanaTokenBlockMessage,
};

// Launchpad program addresses (base58) for Solana token-creation filter.
const SOLANA_PGM_6EF8: &str = "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P";
const SOLANA_PGM_LAN: &str = "LanMV9sAd7wArD4vJFi2qDdfnVhFxYSUg6eADduJ3uj";
const SOLANA_PGM_DBCIJ: &str = "dbcij3LWUppWqq96dh6gJWwBifmcGfLSB5D4DuSMaqN";
const SOLANA_PGM_HEAVEN: &str = "HEAVENoP2qxoeuF8Dj2oT1GHEnu49U5mJYkdeC8BAX2o";
const SOLANA_PGM_MOON: &str = "MoonCVVNZFSYkqNXP6bxHLPL6QQJiMagDL3qcqUQTrG";
const SOLANA_ACCT_BAGSB: &str = "BAGSB9TpGrZxQbEsrEznv5jXXdwyP6AXerN8aVRiAmcv";

/// Parsed ABI topic: token creation = Call.Create == true; pair created = log.Parsed.Signature.Name == "PairCreated"
/// For BSC, meme migration is labeled "BSC fourmeme migration" (same condition: PairCreated + Transaction.To).
pub fn evm_parsed_abi(block: &EvmParsedAbiBlockMessage, encoding: ChainEncoding, chain: &str) {
    for (_tx_idx, tx) in block.transactions.iter().enumerate() {
        // Token creation: any call with Create == true
        for call in &tx.calls {
            if call.header.as_ref().map(|h| h.create).unwrap_or(false) {
                if let Some(ref th) = tx.transaction_header {
                    println!("--- EVM Token creation (Call.Create = true) ---");
                    println!("  TxHash: {}", format_bytes(&th.hash, encoding));
                }
            }
        }
        // Pair created: any log with Parsed.Signature.Name == "PairCreated"
        let mut pair_created = false;
        for call in &tx.calls {
            for log in &call.logs {
                if log.parsed.as_ref().and_then(|p| p.signature.as_ref()).map(|s| s.name == "PairCreated").unwrap_or(false) {
                    pair_created = true;
                    break;
                }
            }
            if pair_created {
                break;
            }
        }
        if pair_created {
            if let Some(ref th) = tx.transaction_header {
                println!("--- EVM PairCreated ---");
                println!("  TxHash: {}", format_bytes(&th.hash, encoding));
            }
        }
        // Meme token creation: Transaction.To == MEME_TOKEN_CREATE_TO and Log.Signature.Name == "TokenCreate"
        let to_matches = tx
            .transaction_header
            .as_ref()
            .map(|th| format_bytes(&th.to, encoding) == MEME_TOKEN_CREATE_TO)
            .unwrap_or(false);
        let mut has_token_create_log = false;
        for call in &tx.calls {
            for log in &call.logs {
                if log.parsed.as_ref().and_then(|p| p.signature.as_ref()).map(|s| s.name == "TokenCreate").unwrap_or(false) {
                    has_token_create_log = true;
                    break;
                }
            }
            if has_token_create_log {
                break;
            }
        }
        if to_matches && has_token_create_log {
            if let Some(ref th) = tx.transaction_header {
                println!("--- EVM Meme token creation (TokenCreate) ---");
                println!("  TxHash: {}", format_bytes(&th.hash, encoding));
            }
        }
        // Meme token migration / BSC fourmeme: Transaction.To == MEME_TOKEN_CREATE_TO and Log.Signature.Name == "PairCreated"
        if to_matches && pair_created {
            if let Some(ref th) = tx.transaction_header {
                if chain.eq_ignore_ascii_case("bsc") {
                    println!("--- BSC fourmeme migration ---");
                } else {
                    println!("--- EVM Meme token migration (PairCreated) ---");
                }
                println!("  TxHash: {}", format_bytes(&th.hash, encoding));
            }
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
        // ----- Set of conditions A: uncomment for token creation on xyz -----
        // if transfer.Currency is first-seen or protocol_name == "xyz" { ... }
        // if /* condition A */ {
        //     println!("--- EVM Token (token creation) ---");
        //     println!("  TxIndex: {}", transfer.transaction_index);
        //     println!("  Sender: {}", format_bytes(&transfer.sender, encoding));
        //     println!("  Receiver: {}", format_bytes(&transfer.receiver, encoding));
        // }
    }
}

/// Solana Dex block: optional filter blocks. Print only tx/signature when matched.
pub fn solana_dex(block: &SolanaDexParsedBlockMessage, _encoding: ChainEncoding) {
    #[allow(unused_variables)]
    for dex_tx in &block.transactions {
        // Print details only when the transaction matches the criteria below.
        // ----- Set of conditions A: uncomment for token creation on xyz -----
        // if solana: e.g. tx has new Currency / mint, or program == xyz
        // if let Some(ref header) = dex_tx.header {
        //     if header.accounts.iter().any(|a| ...) { ... }
        // }
        // if /* condition A */ {
        //     println!("--- Transaction (token creation) ---");
        //     println!("  Index: {}", dex_tx.index);
        //     println!("  Signature: {}", format_bytes(&dex_tx.signature, encoding));
        //     if let Some(status) = &dex_tx.status {
        //         println!("  Status: success={}", status.success);
        //     }
        // }

    
    }
}

/// Solana Token block: placeholder filter blocks. Print only tx/signature when matched.
pub fn solana_token(block: &SolanaTokenBlockMessage, _encoding: ChainEncoding) {
    #[allow(unused_variables)]
    for _tx in &block.transactions {
        // ----- Set of conditions A: uncomment for token creation on xyz -----
        // ----- Set of conditions B: uncomment for migration of abcd -----
    }
}

/// Returns true if this instruction is a PumpFun migrate (Program 6EF8, logs include "Migrate").
fn is_pumpfun_migrate_instruction(
    inst: &crate::protos::solana::ParsedIdlInstruction,
    encoding: ChainEncoding,
) -> bool {
    let program = match &inst.program {
        Some(p) => p,
        None => return false,
    };
    if format_bytes(&program.address, encoding) != SOLANA_PGM_6EF8 {
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
    let addr = format_bytes(&program.address, encoding);
    let method = program.method.as_str();
    // 6EF8: create | create_v2
    if addr == SOLANA_PGM_6EF8 && (method == "create" || method == "create_v2") {
        return true;
    }
    // Lan: initialize_v2
    if addr == SOLANA_PGM_LAN && method == "initialize_v2" {
        return true;
    }
    // dbcij3: initialize_virtual_pool_with_spl_token (plain or with account conditions)
    if addr == SOLANA_PGM_DBCIJ && method == "initialize_virtual_pool_with_spl_token" {
        let has_bagsb = inst
            .accounts
            .iter()
            .any(|a| format_bytes(&a.address, encoding) == SOLANA_ACCT_BAGSB);
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
    if addr == SOLANA_PGM_HEAVEN && method == "create_standard_liquidity_pool" {
        return true;
    }
    // Moon: tokenMint
    if addr == SOLANA_PGM_MOON && method == "tokenMint" {
        return true;
    }
    false
}

/// Solana ParsedIdl block: launchpad token-creation filter. Print only tx hash when matched.
pub fn solana_parsed_idl(block: &SolanaParsedIdlBlockMessage, encoding: ChainEncoding) {
    for tx in &block.transactions {
        let success = tx
            .status
            .as_ref()
            .map(|s| s.success)
            .unwrap_or(false);
        if !success {
            continue;
        }
        let is_launchpad = tx
            .parsed_idl_instructions
            .iter()
            .any(|inst| is_launchpad_token_creation_instruction(inst, encoding));
        if is_launchpad {
            println!("--- Solana launchpad token creation ---");
            println!("  TxHash: {}", format_bytes(&tx.signature, encoding));
        }
        // Migration to pumpswap from pumpfun: Program 6EF8... and instruction logs include "Migrate"
        let is_pumpfun_migrate = tx
            .parsed_idl_instructions
            .iter()
            .any(|inst| is_pumpfun_migrate_instruction(inst, encoding));
        if is_pumpfun_migrate {
            println!("--- Solana migration to pumpswap from pumpfun ---");
            println!("  TxHash: {}", format_bytes(&tx.signature, encoding));
        }
    }
}

/// Solana raw BlockMessage: block with transactions and rewards. Print only tx/signature when matched.
pub fn solana_block(_block: &SolanaBlockMessage, _encoding: ChainEncoding) {
    // BlockMessage: block with transactions and rewards
}
