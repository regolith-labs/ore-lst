use serde::{Deserialize, Serialize};
use steel::*;

use super::OreLstAccount;

/// On-chain account representing the ORE LST vault.
///
/// The vault PDA controls the staked ORE position and authorizes
/// minting/burning of stORE tokens.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Vault {}

account!(OreLstAccount, Vault);
