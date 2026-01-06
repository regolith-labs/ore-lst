use serde::{Deserialize, Serialize};
use steel::*;

use super::OreLstAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Vault {}

account!(OreLstAccount, Vault);
