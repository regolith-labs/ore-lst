mod vault;

pub use vault::*;

use crate::consts::*;

use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum OreLstAccount {
    Vault = 100,
}

pub fn vault_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[VAULT], &crate::ID)
}
