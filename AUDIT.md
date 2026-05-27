# ORE LST (stORE) Security Audit Report

**Contract:** `LStwN2E5Uw6MCtuxHRLhy8RY9hxqW2XRpLzettb696y`
**Version:** 0.2.3
**Date:** 2026-05-26
**Scope:** All on-chain program logic (`program/src/`) and API definitions (`api/src/`)
**Context:** Pre-freeze audit (upgrade authority revocation)

---

## Summary

The ORE LST program is a liquid staking wrapper for ORE tokens. Users deposit ORE via `wrap` (receiving stORE), and redeem via `unwrap` (burning stORE). Staking yield is auto-compounded by calling `compound`. The program holds a single vault PDA that controls the staked ORE position and serves as the stORE mint authority.

**Overall assessment:** The contract is well-structured with a minimal attack surface. No critical or high severity issues were found. The core wrap/unwrap math is correct, overflow checks are enabled, and the program has no admin backdoors post-initialization -- making it a good candidate for freezing.

| Severity | Count |
|----------|-------|
| Critical | 0     |
| High     | 0     |
| Medium   | 0     |
| Low      | 0     |
| Info     | 10    |

---

## Findings

### LOW

#### L-1: `store_mint_info` not explicitly validated against `STORE_MINT_ADDRESS` in wrap and unwrap

**Files:** `program/src/wrap.rs:24`, `program/src/unwrap.rs:25`

In `wrap.rs`, `store_mint_info` is checked with `.as_mint()?` (valid SPL mint) but is never checked with `.has_address(&STORE_MINT_ADDRESS)?` as it is in `initialize.rs:22`. The same applies to `unwrap.rs`.

**Impact:** If a fake mint were passed, the `store_mint.supply()` value used in the ratio calculation would be wrong. The transaction would ultimately revert because `mint_to_signed` (wrap) or `burn` (unwrap) would fail at the SPL Token program level -- the vault PDA is only the mint authority for the real stORE mint, and the sender's ATA is validated against `STORE_MINT_ADDRESS` when it already exists. So no funds are at risk. However, this relies on implicit downstream validation rather than explicit upfront checks.

**Recommendation:** Add `store_mint_info.has_address(&STORE_MINT_ADDRESS)?` in both `wrap.rs` and `unwrap.rs` for defense-in-depth.

---

#### L-2: `vault_info` not explicitly checked against expected PDA address

**Files:** `program/src/wrap.rs:31`, `program/src/unwrap.rs:32`, `program/src/compound.rs:24`

In all three user-facing instructions, `vault_info` is validated as `.as_account_mut::<Vault>(&ore_lst_api::ID)?` which checks the account owner and discriminator. However, it is not explicitly verified to be at the expected PDA address (`vault_pda().0`).

**Impact:** Since only the ore-lst program can create accounts it owns, and only one Vault is ever created (during `initialize`), there can only be one valid Vault account on-chain. Additionally, `invoke_signed` with seeds `&[VAULT]` would fail if the account isn't the correct PDA. No funds at risk, but an explicit PDA address check would make validation self-documenting.

**Recommendation:** Add `vault_info.has_address(&vault_pda().0)?` or verify the PDA derivation explicitly.

---

#### L-3: Metadata `is_mutable` set to `true`

**File:** `program/src/initialize.rs:53`

The stORE token metadata is created with `is_mutable: true`. The update authority is the vault PDA.

**Impact:** After freezing the program, no instruction exists to call the Metaplex `UpdateMetadata` CPI with the vault PDA as signer. So the metadata is effectively immutable post-freeze. However, the `is_mutable` flag being true is a cosmetic concern that could cause confusion for integrators or auditors inspecting the metadata on-chain.

**Recommendation:** Set `is_mutable: false` before freezing, or add a one-time instruction to flip it to immutable, then freeze.

---

### INFO

#### I-1: Rounding in fixed-point arithmetic slightly favors the pool

**Files:** `program/src/wrap.rs:136`, `program/src/unwrap.rs:118`

The `Numeric` fixed-point type (from the `steel` crate) is used for ratio calculations. Truncation during `.to_u64()` means:
- **Wrap:** `mint_amount` rounds down -- user may receive slightly less stORE than the exact proportional amount.
- **Unwrap:** `redeemable_amount` rounds down -- user may receive slightly less ORE than the exact proportional amount.

This rounding consistently favors the pool (and thus remaining stORE holders), which is the standard and correct behavior for vault-style LST designs. The dust amounts are negligible (< 1 token unit).

---

#### I-2: No test suite

No automated tests exist for the on-chain program. The `program/Cargo.toml` has `rand` as a dev-dependency but no test files are present.

For a program being permanently frozen, comprehensive test coverage (unit tests, integration tests, and ideally fuzz tests on the ratio math) would increase confidence in correctness.

---

#### I-3: Compound in every wrap/unwrap prevents ratio manipulation

**Files:** `program/src/wrap.rs:54-92`, `program/src/unwrap.rs:55-92`

Both `wrap` and `unwrap` perform a full claim + compound before calculating the exchange ratio. This is an important design property: it means the ratio always reflects the latest state, and an attacker cannot front-run with a `compound` call to manipulate the ratio.

This is a positive security property and is well-implemented.

---

#### I-4: Compound is permissionless

**File:** `program/src/compound.rs:16`

Any signer can call `compound`. This is by design -- compounding benefits all stORE holders equally. There is no economic incentive to withhold compounding since wrap/unwrap also compound. No issue, but worth documenting.

---

#### I-5: No fee mechanism

The program charges no fees on wrap, unwrap, or compound. After freezing, no fee can ever be added. This is a strong guarantee for users.

---

#### I-6: No admin functions post-initialization

The only admin-gated instruction is `Initialize`, which is protected by a hardcoded `ADMIN_ADDRESS` check (`program/src/initialize.rs:9,19`) and can only be called once (vault account must be empty). After initialization, all instructions are permissionless. There are no upgrade, pause, migrate, or emergency withdrawal functions.

This is ideal for a program being frozen -- there are no admin backdoors.

---

#### I-7: Re-initialization is properly prevented

**File:** `program/src/initialize.rs:24`

The `vault_info.is_empty()?` check ensures `Initialize` can only succeed when the vault account does not yet exist. Since the vault is a PDA owned by the program and the program has no close/delete instruction, re-initialization is permanently blocked.

---

#### I-8: Overflow checks enabled in all profiles

**File:** `Cargo.toml:46-49`

Both `[profile.release]` and `[profile.dev]` set `overflow-checks = true`. This means arithmetic overflow will panic rather than silently wrap, preventing a class of integer overflow vulnerabilities.

---

#### I-9: `metadata_program` not validated in Initialize

**File:** `program/src/initialize.rs:35`

The Metaplex metadata program account is not checked with `.is_program(&mpl_token_metadata::ID)?`. Since `Initialize` is admin-only and one-time, the admin would pass the correct program. No risk, but inconsistent with the validation pattern used for other program accounts.

---

#### I-10: Security contact information included

**File:** `program/src/lib.rs:37-44`

The program includes a `security_txt!` macro with contact information, source code link, and security policy URL. This follows Solana security best practices.

---

## Architecture Review

### Instruction Flow

```
Initialize (admin, one-time)
  -> Create metadata, vault PDA, vault token account, stake account

Wrap (permissionless)
  -> Claim yield -> Compound yield -> Calculate ratio -> Transfer ORE -> Deposit -> Mint stORE

Unwrap (permissionless)
  -> Claim yield -> Compound yield -> Calculate ratio -> Burn stORE -> Withdraw -> Transfer ORE

Compound (permissionless)
  -> Claim yield -> Deposit yield
```

### Exchange Rate Math

**Wrap:** `stORE_minted = ORE_deposited * (stORE_supply / staked_ORE_balance)`
**Unwrap:** `ORE_redeemed = stORE_burned * (staked_ORE_balance / stORE_supply)`

The ratio is calculated AFTER compounding yield but BEFORE the user's deposit/withdrawal, ensuring fair pricing.

### Account Validation Summary

| Account | wrap | unwrap | compound | initialize |
|---------|------|--------|----------|------------|
| signer | is_signer | is_signer | is_signer | is_signer + ADMIN_ADDRESS |
| ore_mint | has_address + as_mint | has_address + as_mint | has_address + as_mint | has_address + as_mint |
| store_mint | as_mint only | as_mint only | N/A | has_address + as_mint |
| vault | as_account (owner+disc) | as_account (owner+disc) | as_account (owner+disc) | is_empty + is_writable |
| stake | as_account + authority check | as_account + authority check | as_account + authority check | N/A |
| system_program | is_program | is_program | is_program | is_program |
| token_program | is_program | is_program | is_program | is_program |
| assoc_token_prog | is_program | is_program | is_program | is_program |
| ore_stake_prog | is_program | is_program | is_program | is_program |
| metadata_program | N/A | N/A | N/A | not validated |

### Freeze Readiness Checklist

- [x] No admin backdoors post-initialization
- [x] No upgrade/migrate instructions
- [x] Re-initialization prevented
- [x] Overflow checks enabled
- [x] Permissionless compound prevents ratio manipulation
- [x] Rounding favors the pool (standard LST behavior)
- [x] Exchange rate math is correct
- [x] Security contact information present
- [ ] store_mint_info should be explicitly validated (L-1)
- [ ] vault PDA address should be explicitly validated (L-2)
- [ ] Metadata should be set to immutable before freeze (L-3)
- [ ] No automated test suite (I-2)

---

## Conclusion

The ORE LST program is a clean, minimal liquid staking wrapper with a small attack surface. The core economic logic (ratio calculation, compound-before-price) is correctly implemented. All four low-severity findings are defense-in-depth improvements where implicit CPI-level validation currently prevents exploitation. No critical, high, or medium issues were identified.

**The program is suitable for freezing**, with the recommendation to address L-1 through L-3 before revoking upgrade authority, and to add test coverage (I-2) for additional confidence.
