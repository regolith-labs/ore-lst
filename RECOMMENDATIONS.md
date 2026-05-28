# Pre-Freeze Recommendations

Full audit of all source files, dependencies, and on-chain assumptions. Only actionable items listed.

---

## Blocking (must complete before revoking upgrade authority)

### 1. Verify ore-stake upgrade authority is revoked

The ore-lst program passes the vault PDA as a signer to ore-stake via CPI. Solana's signer transitivity means ore-stake can use that signer authority to call any program (SPL token, Metaplex, etc.) on the vault's behalf. If ore-stake is upgraded to a malicious version post-freeze, it could drain all vault ORE, mint unlimited stORE, or modify token metadata.

**Action**: Confirm on-chain that ore-stake (`orestake...`) has no upgrade authority, or document the trust assumption explicitly.

### 2. Verify ore-mint upgrade authority is revoked

Same rationale as above — ore-mint controls the ORE token. If it were upgraded maliciously, it could affect all downstream programs.

**Action**: Confirm on-chain or document the trust assumption.

### 3. Verify program binary matches source

**Action**: Run a verifiable build and confirm the deployed binary matches this repository at the frozen commit.

### 4. Verify token metadata on-chain

**Action**: Confirm the on-chain metadata reads name: "Staked ORE", symbol: "stORE", and the URI resolves correctly.

---

## Recommended (should do, no code change required)

### 5. Set metadata `is_mutable` to false

`init.rs:51` sets `is_mutable: true` on the Metaplex metadata. Post-freeze, there is no instruction that can update metadata, so it is effectively immutable — but it is not marked as such on-chain. A mutable metadata flag could confuse integrators or indexers.

**Action**: Before revoking upgrade authority, either:
- (a) Add a one-time instruction to call `UpdateMetadataAccountV2` with `is_mutable: false`, deploy, execute, then freeze. Or:
- (b) Accept and document that metadata is effectively immutable despite the flag.

---

## Cleanup (optional, no security impact)

### 6. Remove unused workspace dependencies

The following are declared in the root `Cargo.toml` `[workspace.dependencies]` but not referenced by any member crate (`api`, `program`, or `cli`):

- `base64`
- `bytemuck_derive`
- `const-crypto`
- `serde_json`
- `solana-account-decoder`
- `solana-address-lookup-table-interface`
- `solana-nostd-keccak`
- `spl-pod`
- `spl-token-metadata-interface`

These don't affect the compiled binary but add noise. Safe to remove.

---

## No action needed

The following were reviewed and require no changes:

- **Exchange rate math** — rounding always favors the pool, zero-output guard prevents dust grief, roundtrip tests confirm no overpayment.
- **Account validation** — all accounts are validated before use; passthrough accounts (vesting, treasury) are validated by the downstream ore-stake program.
- **Data freshness after CPI** — `stake_info` is correctly re-read after compound CPIs; `store_mint` is read before compound but compound does not change stORE supply, so the value is correct.
- **Overflow safety** — `overflow-checks = true` in both release and dev profiles.
- **No unsafe code, no TODOs, no dead code in program sources.**
- **Permissionless compound** — by design, benefits all holders.
- **First-depositor inflation** — mitigated by existing ~39K ORE / ~37K stORE in the vault.
