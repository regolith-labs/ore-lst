# ORE LST (stORE) Security Audit

**Program**: `LStwN2E5Uw6MCtuxHRLhy8RY9hxqW2XRpLzettb696y`
**Version**: 0.2.3
**Date**: 2026-05-27
**Context**: Pre-freeze audit. Upgrade authority will be revoked after this review.

---

## Summary

ORE LST is a liquid staking token (stORE) for ORE on Solana. Users deposit ORE via `wrap` (receiving stORE), and redeem ORE via `unwrap` (burning stORE). Staking yields compound automatically, increasing the ORE redeemable per stORE over time. The program has four instructions: `init`, `wrap`, `unwrap`, and `compound`.

The program is small, well-scoped, and follows sound Solana patterns. No critical or high-severity vulnerabilities were identified. The issues found are design-level considerations and missing guardrails that cannot cause loss of funds under normal operating conditions.

---

## Findings

### HIGH

*None.*

---

### MEDIUM

*None.*

---

### LOW

*None.*

---

#### I-11: Dust Amounts Can Round to Zero Output

**Location**: `program/src/wrap.rs:103-104`, `program/src/unwrap.rs:104-105`

When wrapping a very small amount of ORE and the stORE:ORE ratio is above 1:1 (due to accumulated yield), `calculate_mint_amount` can floor to 0 via `Numeric::to_u64()`. The user's ORE is deposited but they receive 0 stORE. Similarly for unwrap with dust amounts.

Only affects dust-sized transactions relative to the exchange rate. Accepted risk.

---

### INFO

#### I-1: Rounding Always Favors the Pool

Both `calculate_mint_amount` and `calculate_redeem_amount` use `Numeric::to_u64()` which floors (truncates) the result. This means:
- **Wrap**: Users receive slightly fewer stORE than their exact proportional share (pool keeps the dust).
- **Unwrap**: Users receive slightly less ORE than their exact proportional share (pool keeps the dust).

This is the correct and standard rounding direction for vault/pool designs. The rounding error benefits remaining stORE holders.

---

#### I-2: Compound Is Permissionless

The `compound` instruction requires only a signer (any signer). Anyone can trigger yield compounding at any time. This is by design — compounding benefits all stORE holders, so there is no reason to restrict it. Third-party bots or keepers can call this to ensure timely compounding.

---

#### I-3: Overflow Checks Enabled

`Cargo.toml` specifies `overflow-checks = true` for both `[profile.release]` and `[profile.dev]`. Arithmetic overflow will panic (abort the transaction) rather than silently wrapping. This is a strong safety net.

---

#### I-4: No Emergency Pause or Admin Functions

The program has no admin authority, no pause mechanism, no fee extraction, and no governance. After freeze, no party can:
- Pause wrap/unwrap in case of a discovered vulnerability
- Upgrade the program to fix bugs
- Extract fees or redirect funds

This is the explicit design choice for maximizing trustlessness and removing counterparty risk. Users should understand that post-freeze, the program behavior is permanent.

---

#### I-5: External Dependency on ore-stake Program

The ore-lst program delegates all staking operations (deposit, withdraw, claim) to the ore-stake program (`ore-stake-api v0.2.4`). The security of ore-lst depends on:
- ore-stake correctly tracking `Stake.balance` and `Treasury.rewards_factor`
- ore-stake not being upgradeable to a malicious version (or having its own upgrade authority revoked)
- ore-stake's vesting and reward distribution logic being correct

If ore-stake is compromised or has bugs, ore-lst users are affected. The ore-stake program ID is hardcoded and validated (`ore_stake_program.is_program(&ore_stake_api::ID)?`), so it cannot be substituted.

**Recommendation**: Verify that the ore-stake program's upgrade authority is also revoked, or document the trust assumption.

---

#### I-6: Vault Account Stores No State

The `Vault` struct is empty (`struct Vault {}`). It exists solely as a PDA to serve as the signing authority for CPIs (staking, minting, burning). All meaningful state (staked balance, stORE supply) lives in the ore-stake `Stake` account and the stORE mint respectively. This is clean design with a single source of truth for each value, eliminating state synchronization bugs.

---

#### I-7: Store Mint Is Pre-Created Externally

The stORE mint (`sTorERYB6xAZ1SSbwpK3zoK2EEwbBrc7TZAzg1uCGiH`) is a hardcoded constant. It must be created externally before `init` is called, with the vault PDA as its mint authority. The `init` instruction validates the mint exists (`as_mint()?`) and implicitly validates authority (the metadata CPI requires mint authority signature from the vault PDA).

**Recommendation**: Verify on-chain that:
1. The stORE mint authority is the vault PDA and no other entity.
2. The stORE mint freeze authority is `None` (otherwise, an attacker with freeze authority could freeze user token accounts, preventing unwrap).

---

#### I-8: No Integration or End-to-End Test Suite

The repository contains unit tests for `Vault::calculate_mint_amount` and `Vault::calculate_redeem_amount` (in `api/src/state/vault.rs`), which cover edge cases and roundtrip invariants. However, there are no integration tests that exercise the full instruction flow (init, wrap, unwrap, compound) against a local validator or BPF test harness.

---

#### I-9: Security Contact Information Present

The program includes `solana-security-txt` with contact information (email, Discord) and links to the source code and security policy. This is good practice for responsible disclosure.

---

#### I-10: No Programmatic First-Depositor Inflation Guard

The program has no virtual offset or dead-share mechanism to protect against the classic vault share inflation attack (ERC-4626 style). An attacker who is the first depositor could donate ORE directly to `vault_tokens`, call `compound` to inflate `stake.balance` without minting stORE, and cause subsequent depositors' `calculate_mint_amount` to round to 0.

This is not exploitable because the vault is already initialized with existing deposits, making the cost to meaningfully skew the ratio far exceed any profit. Documented here for completeness.

---

## Pre-Freeze Checklist

Before revoking upgrade authority, verify the following on-chain:

- [x] **stORE mint authority** is the vault PDA (`7taXpXz6eqYzscXEi1d1fgwATQMqAR6Nku9pJCjb8gQN`) -- verified on-chain
- [x] **stORE mint freeze authority** is `None` -- verified on-chain
- [x] **Vault PDA** is initialized (8 bytes, owned by `LStwN2E5Uw6MCtuxHRLhy8RY9hxqW2XRpLzettb696y`) -- verified on-chain
- [x] **Stake account** exists with `authority == vault PDA` and balance of ~39,568 ORE (`DfdZYzgLuqRickq57fyb4dX88VgPkhoEs1uuBKdxzaaJ`) -- verified on-chain
- [x] **stORE supply** is ~37,127 stORE (non-trivial, mitigates first-depositor inflation, see I-10) -- verified on-chain
- [ ] **ore-stake program** upgrade authority is also revoked (or trust assumption documented)
- [ ] **ore-mint program** upgrade authority is also revoked (or trust assumption documented)
- [ ] **Token metadata** is correct (name: "Staked ORE", symbol: "stORE")
- [ ] **Program binary** matches the source code in this repository (verifiable build)
