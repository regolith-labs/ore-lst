use steel::*;

/// Instruction discriminators for the ORE LST program.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum OreLstInstruction {
    /// Claims staking rewards and re-stakes them to compound yield.
    Compound = 1,
    /// Burns stORE tokens to withdraw the underlying staked ORE.
    Unwrap = 2,
    /// Deposits ORE into the vault and mints stORE tokens.
    Wrap = 3,

    /// Initializes the vault and stORE mint. Admin-only.
    Initialize = 100,
}

/// Instruction data for [`OreLstInstruction::Initialize`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Initialize {}

/// Instruction data for [`OreLstInstruction::Compound`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Compound {}

/// Instruction data for [`OreLstInstruction::Unwrap`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Unwrap {
    /// Amount of stORE tokens to burn, as little-endian bytes.
    pub amount: [u8; 8],
}

/// Instruction data for [`OreLstInstruction::Wrap`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Wrap {
    /// Amount of ORE tokens to deposit, as little-endian bytes.
    pub amount: [u8; 8],
}

instruction!(OreLstInstruction, Compound);
instruction!(OreLstInstruction, Initialize);
instruction!(OreLstInstruction, Unwrap);
instruction!(OreLstInstruction, Wrap);
