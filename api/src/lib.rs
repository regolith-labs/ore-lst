//! API for the ORE liquid staking token (stORE) program.
//!
//! This crate provides constants, instructions, state definitions, and SDK helpers
//! for interacting with the ORE LST program on Solana.

pub mod consts;
pub mod error;
pub mod instruction;
pub mod sdk;
pub mod state;

/// Re-exports all public types for convenient imports.
pub mod prelude {
    pub use crate::consts::*;
    pub use crate::error::*;
    pub use crate::instruction::*;
    pub use crate::sdk::*;
    pub use crate::state::*;
}

use steel::*;

declare_id!("LStwN2E5Uw6MCtuxHRLhy8RY9hxqW2XRpLzettb696y");
