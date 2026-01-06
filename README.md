# ORE LST

ORE liquid staking token (stORE).


## API
- [`Consts`](api/src/consts.rs) – Program constants.
- [`Error`](api/src/error.rs) – Custom program errors.
- [`Event`](api/src/error.rs) – Custom program events.
- [`Instruction`](api/src/instruction.rs) – Declared instructions and arguments.

## Instructions

- [`Initialize`](program/src/initialize.rs) - Initializes program variables.
- [`Unwrap`](program/src/unwrap.rs) - Burn stORE and withdraw staked ORE.
- [`Wrap`](program/src/wrap.rs) - Stake ORE and mint new stORE.
- [`Compound`](program/src/compound.rs) - Auto-compound yield.


## State
- [`Vault`](api/src/state/vault.rs) - The program authority for minting stORE.


## Tests

To run the test suite, use the Solana toolchain: 

```
cargo test-sbf
```

For line coverage, use llvm-cov:

```
cargo llvm-cov
```
