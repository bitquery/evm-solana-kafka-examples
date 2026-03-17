/// All Solana streaming topics (block, token, dex, parsed_idl) from solana_messages.
pub mod solana {
    include!(concat!(env!("OUT_DIR"), "/solana_messages.rs"));
}

pub mod evm {
    pub mod evm_messages {
        include!(concat!(env!("OUT_DIR"), "/evm_messages.rs"));
    }
}
