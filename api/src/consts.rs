use solana_program::pubkey;
use steel::Pubkey;

/// The seed of the vault account PDA.
pub const VAULT: &[u8] = b"vault";

/// The seed of the store mint account PDA.
pub const MINT: &[u8] = b"MINT";

/// Mint address of the stORE token.
pub const STORE_MINT_ADDRESS: Pubkey = pubkey!("sTorERYB6xAZ1SSbwpK3zoK2EEwbBrc7TZAzg1uCGiH");
