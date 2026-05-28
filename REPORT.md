# ORE LST (stORE) — Pre-Freeze Audit Report

**Program**: `LStwN2E5Uw6MCtuxHRLhy8RY9hxqW2XRpLzettb696y`
**Version**: 0.2.4
**Date**: 2026-05-27
**Scope**: Full codebase review — program, API, CLI. All `.rs` source files, all `Cargo.toml` files.

---

## Architecture Overview

ORE LST is a liquid staking wrapper for ORE on Solana. Users deposit ORE (`wrap`) and receive stORE tokens proportional to their share of the vault's staked position. They burn stORE (`unwrap`) to redeem ORE. Staking yield accrues to the vault, increasing the ORE-per-stORE ratio over time. A permissionless `compound` instruction claims and re-stakes yield.

**Instructions**: `init`, `wrap`, `unwrap`, `compound`
**State**: A single empty `Vault` PDA (authority only, no stored fields). All real state lives in the ore-stake `Stake` account and the stORE SPL mint.
**External dependencies**: ore-stake (staking), ore-mint (ORE token), mpl-token-metadata, SPL token program.

---

## Findings

### CRITICAL

*None.*

---

### HIGH

*None.*

---

### MEDIUM

#### M-1: CPI Signer Privilege Transitivity — ore-stake Trust Assumption

**Location**: Every `invoke_signed` CPI to ore-stake (compound.rs:32-69, wrap.rs:56-132, unwrap.rs:57-135)

When ore-lst invokes ore-stake with the vault PDA as signer, ore-stake receives full signer authority over the vault PDA for that CPI. In Solana's CPI model, signer authority is transitive — ore-stake could further CPI to the SPL token program (or any program) using the vault PDA as signer. This means:

- If ore-stake is upgraded to a malicious version, it could **drain vault_tokens** by transferring ORE out.
- It could **mint unlimited stORE** by calling mint_to with the vault PDA as mint authority.
- It could **update token metadata** since the vault PDA is the metadata update authority.

This is the single most important risk surface for this program. ore-lst is secure in isolation, but its security is upper-bounded by ore-stake's immutability.

**Status**: Documented in existing AUDIT.md (I-5). The pre-freeze checklist item "ore-stake program upgrade authority is also revoked" is still unchecked.

**Recommendation**: Do not revoke ore-lst's upgrade authority until ore-stake's upgrade authority is confirmed revoked on-chain. This is a hard prerequisite.

---

### LOW

#### L-1: Zero-Amount Transactions Are Not Rejected

**Location**: `program/src/wrap.rs:102-106`, `program/src/unwrap.rs:103-108`

Both wrap and unwrap accept `amount = 0`. The guard `if amount > 0 && mint_amount == 0` short-circuits only when the *output* is zero with a *positive* input. When `amount = 0`:
- The compound CPIs still execute (claim + deposit), costing ~200K+ compute.
- A transfer of 0 tokens and mint/burn of 0 tokens are issued.

This is not exploitable (the caller pays the fees), but it does allow spam that triggers unnecessary compound operations. The compound itself is beneficial, so this is net-neutral.

**Recommendation**: Optional — add `if amount == 0 { return Err(...); }` at the top of wrap/unwrap to save compute for accidental zero calls.

---

#### L-2: Repeated PDA Derivation in Hot Path

**Location**: Across compound.rs, wrap.rs, unwrap.rs — every `invoke_signed` call with `&[VAULT]` seeds, plus `vault_pda().0`/`.1` in validation.

Steel's `invoke_signed` helper internally calls `find_program_address` to derive the bump for each CPI. Wrap and unwrap each make 3 CPIs to ore-stake plus 1 token operation, resulting in ~4 PDA derivations per instruction. `find_program_address` iterates bump values and hashes, costing ~1,500 CU each.

Not a vulnerability, but totals ~6,000 CU of avoidable work per instruction. With the existing 1.4M CU budget this is fine, but it's dead weight.

**Recommendation**: Accept as-is. The Steel framework handles this internally and the program is about to freeze.

---

#### L-3: Unused `spl-token-2022` Dependency

**Location**: `api/Cargo.toml:21`

`spl-token-2022` is listed as a dependency in the API crate but never imported or used in any source file. This adds unnecessary compile-time weight and could confuse consumers into thinking Token-2022 is supported.

**Recommendation**: Remove before freeze if possible. Purely cosmetic for the on-chain program binary (it's API-crate only), but affects downstream consumers.

---

#### L-4: Passthrough Accounts Not Validated Locally

**Location**: `vesting_info` in compound.rs:9, wrap.rs:13, unwrap.rs:13; `metadata_info`, `stake_info`, `stake_tokens_info`, `treasury_info` in init.rs:11

Several accounts are passed directly to CPI targets without local validation. For example, `vesting_info` is never checked by ore-lst — it's passed straight to ore-stake which validates it.

This is safe because the downstream program (ore-stake) will reject invalid accounts. However, defense-in-depth validation would produce clearer error messages and catch issues earlier.

**Recommendation**: Accept. The downstream programs enforce correctness.

---

### INFORMATIONAL

#### I-1: Rounding Direction Is Correct

Both `calculate_mint_amount` and `calculate_redeem_amount` truncate via `Numeric::to_u64()` (floor). This means:
- Wrap: user gets slightly fewer stORE (vault keeps the dust)
- Unwrap: user gets slightly less ORE (vault keeps the dust)

This is the correct and industry-standard rounding direction. The roundtrip tests in `vault.rs:146-193` verify `redeemed <= deposit`, confirming no overpayment.

---

#### I-2: Zero-Output Guard Prevents Dust Grief

`wrap.rs:104-106` and `unwrap.rs:106-108` return `StoreError::OutputZero` when a positive input produces zero output. This prevents a user from accidentally losing funds to rounding. Good.

---

#### I-3: Overflow Checks Enabled

`Cargo.toml:47-50` enables `overflow-checks = true` for both release and dev profiles. All arithmetic panics (aborts transaction) on overflow instead of wrapping. Strong safety net.

---

#### I-4: Compound Is Permissionless (By Design)

Any signer can call `compound`. This benefits all stORE holders by claiming and re-staking yield. There is no incentive to withhold compounding. Third-party keepers/bots can ensure timely compounding.

---

#### I-5: No Admin, Pause, or Upgrade Path Post-Freeze

After upgrade authority is revoked:
- No pause mechanism exists for emergency response.
- No fee extraction or governance is possible.
- No metadata updates are possible (vault PDA is update authority, but no instruction invokes metadata update).

This maximizes trustlessness. Users must understand that post-freeze behavior is permanent — including any undiscovered bugs.

---

#### I-6: Metadata Marked Mutable But Effectively Immutable

`init.rs:51`: `is_mutable: true` is set on the Metaplex metadata. The vault PDA is the update authority. However, post-freeze there is no instruction in the program that invokes `UpdateMetadataAccountV2` or similar. Since the vault PDA can only sign via ore-lst instructions (scoped to this program's seeds), the metadata is effectively immutable after freeze.

Setting `is_mutable: false` would make this explicit, but it would require an additional instruction or a pre-freeze metadata update.

**Recommendation**: Consider calling `UpdateMetadataAccountV2` with `is_mutable: false` before revoking upgrade authority, to make the immutability explicit on-chain. Otherwise, accept as-is — it's non-exploitable.

---

#### I-7: First-Depositor Inflation Attack — Mitigated by Existing State

The program has no virtual offset or dead-share mechanism (a la ERC-4626). In theory, the first depositor could donate ORE and compound to inflate `stake.balance` without minting stORE, causing subsequent depositors' mint amounts to round to zero.

This is not exploitable because:
1. The vault already has ~39,568 ORE staked and ~37,127 stORE in circulation.
2. The zero-output guard (I-2) would reject such transactions.
3. The cost to meaningfully skew the ratio now far exceeds any profit.

---

#### I-8: Account Data Freshness After CPI Is Correct

In wrap.rs and unwrap.rs, `stake_info` is re-read *after* the compound CPIs (wrap.rs:97-99, unwrap.rs:98-100) to get the updated balance. `store_mint` is read *before* the compound CPIs (wrap.rs:22-24, unwrap.rs:23-25) but this is correct because compound does not mint or burn stORE — it only affects `stake.balance`.

The data flow is sound: compound changes stake balance (re-read), doesn't change stORE supply (no re-read needed).

---

#### I-9: No Integration Tests

The codebase has unit tests for the exchange rate math (`vault.rs:46-194`) with good coverage of edge cases and roundtrip invariants. However, there are no integration tests exercising the full instruction flow (init → wrap → compound → unwrap) against a local validator.

For a frozen program, the on-chain behavior is the ultimate test. But the absence of integration tests means any subtle interaction between instructions (e.g., CPI ordering, account data freshness) was verified only by manual testing and code review.

---

#### I-10: CLI Is Dev-Only, Not Security-Relevant

The CLI (`cli/src/main.rs`) is marked `publish = false` and uses hardcoded compute budgets (1.4M CU, 1M microlamport priority fee). It reads secrets from environment variables and `.unwrap()`s liberally. This is appropriate for a developer/operator tool and is not part of the on-chain attack surface.

---

#### I-11: Security Contact Information Present

`program/src/lib.rs:34-41` includes `solana-security-txt` with email, Discord, source code URL, and security policy link. Good practice.

---

## Code Quality

| Aspect | Assessment |
|--------|-----------|
| **Unsafe code** | None. Zero `unsafe` blocks in the entire codebase. |
| **TODO/FIXME/HACK** | None. Clean codebase. |
| **Dead code** | `spl-token-2022` dependency unused (L-3). No dead Rust code. |
| **Naming** | Clear and consistent. `wrap`/`unwrap` map directly to deposit/withdraw semantics. |
| **Error handling** | Program returns `ProgramError` via Steel macros. Custom `StoreError::OutputZero` for the zero-output case. No panicking in program code. |
| **Comments/docs** | API crate is well-documented with doc comments on all public items. Program crate has brief but adequate comments. |
| **Patterns** | Consistent account validation pattern: destructure → validate → CPI. Repeated across all instructions. |
| **Abstraction** | Appropriately minimal. Vault struct is empty by design — no unnecessary state. No over-abstraction. |
| **Test coverage** | Unit tests cover exchange rate math thoroughly, including edge cases and invariant checks. No integration tests. |

---

## Pre-Freeze Action Items

| # | Item | Status | Priority |
|---|------|--------|----------|
| 1 | Confirm ore-stake upgrade authority is revoked | Unchecked | **Blocking** |
| 2 | Confirm ore-mint upgrade authority is revoked | Unchecked | **Blocking** |
| 3 | Verify on-chain: stORE mint authority == vault PDA | Verified | Done |
| 4 | Verify on-chain: stORE mint freeze authority == None | Verified | Done |
| 5 | Verify on-chain: token metadata name/symbol correct | Unchecked | High |
| 6 | Verify on-chain: program binary matches source (verifiable build) | Unchecked | High |
| 7 | Consider setting metadata `is_mutable: false` (I-6) | Not done | Optional |
| 8 | Consider removing `spl-token-2022` from api/Cargo.toml (L-3) | Not done | Optional |

---

## Conclusion

The program is clean, minimal, and correctly implemented for its purpose. No critical or high-severity vulnerabilities were found in the ore-lst code itself. The most significant risk is **M-1**: the transitive CPI signer trust assumption on ore-stake. If ore-stake remains upgradeable after ore-lst is frozen, the entire stORE position could be drained by a compromised ore-stake upgrade. This is the one blocking item before freeze.

The codebase quality is high — no unsafe code, no TODOs, no dead code paths, clear naming, consistent patterns, and appropriate test coverage for the mathematical core. The low-severity findings are all optional improvements that don't affect security.
