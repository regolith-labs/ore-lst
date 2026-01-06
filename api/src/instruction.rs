use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum OreLstInstruction {
    // User
    Compound = 1,
    Unwrap = 2,
    Wrap = 3,

    // Admin
    Initialize = 100,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Initialize {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Compound {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Unwrap {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Wrap {
    pub amount: [u8; 8],
}

instruction!(OreLstInstruction, Compound);
instruction!(OreLstInstruction, Initialize);
instruction!(OreLstInstruction, Unwrap);
instruction!(OreLstInstruction, Wrap);
