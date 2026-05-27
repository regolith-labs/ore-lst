use serde::{Deserialize, Serialize};
use steel::*;

use super::OreLstAccount;

/// On-chain account representing the ORE LST vault.
///
/// The vault PDA controls the staked ORE position and authorizes
/// minting/burning of stORE tokens.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Vault {}

account!(OreLstAccount, Vault);

impl Vault {
    /// Calculates the amount of stORE to mint for a given ORE deposit.
    pub fn calculate_mint_amount(
        ore_amount: u64,
        stake_balance: u64,
        store_supply: u64,
    ) -> u64 {
        let ratio = if stake_balance == 0 || store_supply == 0 {
            Numeric::from_u64(1)
        } else {
            Numeric::from_fraction(store_supply, stake_balance)
        };
        (Numeric::from_u64(ore_amount) * ratio).to_u64()
    }

    /// Calculates the amount of ORE redeemable for a given stORE burn.
    pub fn calculate_redeem_amount(
        store_amount: u64,
        stake_balance: u64,
        store_supply: u64,
    ) -> u64 {
        let ratio = if stake_balance == 0 || store_supply == 0 {
            Numeric::from_u64(1)
        } else {
            Numeric::from_fraction(stake_balance, store_supply)
        };
        (Numeric::from_u64(store_amount) * ratio).to_u64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- calculate_mint_amount --

    #[test]
    fn test_mint_first_deposit() {
        // First deposit: 1:1 ratio
        assert_eq!(Vault::calculate_mint_amount(1_000, 0, 0), 1_000);
    }

    #[test]
    fn test_mint_equal_ratio() {
        // No yield accrued: 1:1 ratio
        assert_eq!(Vault::calculate_mint_amount(1_000, 5_000, 5_000), 1_000);
    }

    #[test]
    fn test_mint_after_yield() {
        // Yield doubled the balance: 1 ORE = 0.5 stORE
        assert_eq!(Vault::calculate_mint_amount(1_000, 10_000, 5_000), 500);
    }

    #[test]
    fn test_mint_zero_amount() {
        assert_eq!(Vault::calculate_mint_amount(0, 10_000, 5_000), 0);
    }

    #[test]
    fn test_mint_one_token_unit() {
        // Smallest possible deposit after yield
        // ratio = 5_000 / 10_000 = 0.5, so 1 * 0.5 = 0 (rounds down)
        assert_eq!(Vault::calculate_mint_amount(1, 10_000, 5_000), 0);
    }

    #[test]
    fn test_mint_rounding_favors_pool() {
        // 3 * (5_000 / 10_000) = 1.5, should round down to 1
        assert_eq!(Vault::calculate_mint_amount(3, 10_000, 5_000), 1);
    }

    #[test]
    fn test_mint_large_values() {
        let balance = 1_000_000_000_000u64; // 1T
        let supply = 500_000_000_000u64;    // 500B
        let deposit = 1_000_000_000u64;     // 1B
        // ratio = 0.5, expected = 500M
        assert_eq!(
            Vault::calculate_mint_amount(deposit, balance, supply),
            500_000_000
        );
    }

    // -- calculate_redeem_amount --

    #[test]
    fn test_redeem_equal_ratio() {
        // No yield: 1:1
        assert_eq!(Vault::calculate_redeem_amount(1_000, 5_000, 5_000), 1_000);
    }

    #[test]
    fn test_redeem_after_yield() {
        // Yield doubled balance: 1 stORE = 2 ORE
        assert_eq!(Vault::calculate_redeem_amount(1_000, 10_000, 5_000), 2_000);
    }

    #[test]
    fn test_redeem_zero_amount() {
        assert_eq!(Vault::calculate_redeem_amount(0, 10_000, 5_000), 0);
    }

    #[test]
    fn test_redeem_one_token_unit() {
        // 1 * (10_000 / 5_000) = 2
        assert_eq!(Vault::calculate_redeem_amount(1, 10_000, 5_000), 2);
    }

    #[test]
    fn test_redeem_rounding_favors_pool() {
        // 1 * (10_000 / 3_000) = 3.333..., should round down to 3
        assert_eq!(Vault::calculate_redeem_amount(1, 10_000, 3_000), 3);
    }

    #[test]
    fn test_redeem_full_supply() {
        // Redeeming all stORE should return all staked ORE
        assert_eq!(Vault::calculate_redeem_amount(5_000, 10_000, 5_000), 10_000);
    }

    #[test]
    fn test_redeem_first_deposit_fallback() {
        // Edge: supply=0, balance=0 -> 1:1 fallback
        assert_eq!(Vault::calculate_redeem_amount(1_000, 0, 0), 1_000);
    }

    // -- roundtrip invariants --

    #[test]
    fn test_roundtrip_no_yield() {
        // Wrap then unwrap at same ratio: should get back <= original
        let balance = 10_000u64;
        let supply = 10_000u64;
        let deposit = 1_000u64;

        let minted = Vault::calculate_mint_amount(deposit, balance, supply);
        let redeemed =
            Vault::calculate_redeem_amount(minted, balance + deposit, supply + minted);
        assert!(redeemed <= deposit);
    }

    #[test]
    fn test_roundtrip_with_yield() {
        // Wrap, yield accrues, unwrap: should get back more than deposited
        let balance = 10_000u64;
        let supply = 10_000u64;
        let deposit = 1_000u64;

        let minted = Vault::calculate_mint_amount(deposit, balance, supply);
        // Simulate yield: balance grows by 20%
        let new_balance = (balance + deposit) * 120 / 100;
        let new_supply = supply + minted;
        let redeemed = Vault::calculate_redeem_amount(minted, new_balance, new_supply);
        assert!(redeemed > deposit);
    }

    #[test]
    fn test_roundtrip_rounding_never_overpays() {
        // Test many amounts: redeem should never exceed proportional share
        for deposit in [1, 2, 3, 7, 99, 1000, 999_999] {
            for (balance, supply) in [(1000, 1000), (7777, 3333), (10000, 1)] {
                let minted = Vault::calculate_mint_amount(deposit, balance, supply);
                if minted == 0 {
                    continue;
                }
                let new_balance = balance + deposit;
                let new_supply = supply + minted;
                let redeemed =
                    Vault::calculate_redeem_amount(minted, new_balance, new_supply);
                assert!(
                    redeemed <= deposit,
                    "Overpaid: deposit={deposit}, balance={balance}, supply={supply}, \
                     minted={minted}, redeemed={redeemed}"
                );
            }
        }
    }
}
