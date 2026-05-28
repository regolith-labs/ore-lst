use steel::*;

/// Errors returned by the ORE LST program.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum StoreError {
    #[error("Output amount is zero")]
    OutputZero = 0,
}

error!(StoreError);
