mod vault;

pub use vault::*;

use crate::consts::*;

use steel::*;

/// Account discriminators for the ORE LST program.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum OreLstAccount {
    /// The vault account holding staked ORE state.
    Vault = 100,
}

/// Derives the vault PDA address and bump seed.
pub fn vault_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[VAULT], &crate::ID)
}
