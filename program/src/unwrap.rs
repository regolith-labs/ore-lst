use ore_api::{
    consts::MINT_ADDRESS,
    state::{Stake, Treasury},
};
use ore_lst_api::prelude::*;
use steel::*;

/// Withdraws ORE from the stake account and burns stORE.
pub fn process_unwrap(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Unwrap::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, payer_info, sender_ore_info, sender_store_info, ore_mint_info, store_mint_info, stake_info, stake_tokens_info, treasury_info, treasury_tokens_info, vault_info, vault_tokens_info, system_program, token_program, associated_token_program, ore_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    payer_info.is_signer()?;
    let sender_store =
        sender_store_info.as_associated_token_account(signer_info.key, &STORE_MINT_ADDRESS)?;
    ore_mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    let store_mint = store_mint_info.as_mint()?;
    stake_info
        .as_account::<Stake>(&ore_api::ID)?
        .assert(|s| s.authority == *vault_info.key)?;
    stake_tokens_info.as_associated_token_account(stake_info.key, &MINT_ADDRESS)?;
    treasury_info.as_account::<Treasury>(&ore_api::ID)?;
    treasury_tokens_info.as_associated_token_account(treasury_info.key, &MINT_ADDRESS)?;
    vault_info.as_account_mut::<Vault>(&ore_lst_api::ID)?;
    vault_tokens_info.as_associated_token_account(vault_info.key, &MINT_ADDRESS)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;
    ore_program.is_program(&ore_api::ID)?;

    // Create sender ore info.
    if sender_ore_info.data_is_empty() {
        create_associated_token_account(
            payer_info,
            signer_info,
            sender_ore_info,
            ore_mint_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        sender_ore_info.as_associated_token_account(signer_info.key, &MINT_ADDRESS)?;
    }

    // Claim yield.
    invoke_signed(
        &ore_api::sdk::claim_yield(*vault_info.key, u64::MAX),
        &[
            vault_info.clone(),
            ore_mint_info.clone(),
            vault_tokens_info.clone(),
            stake_info.clone(),
            treasury_info.clone(),
            treasury_tokens_info.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
        &ore_lst_api::ID,
        &[VAULT],
    )?;

    // Compound yield into vault.
    invoke_signed(
        &ore_api::sdk::deposit(*vault_info.key, *signer_info.key, u64::MAX, 0),
        &[
            vault_info.clone(),
            signer_info.clone(),
            ore_mint_info.clone(),
            vault_tokens_info.clone(),
            stake_info.clone(),
            stake_tokens_info.clone(),
            treasury_info.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
        &ore_lst_api::ID,
        &[VAULT],
    )?;

    // Parse stake account.
    let stake = stake_info
        .as_account::<Stake>(&ore_api::ID)?
        .assert(|s| s.authority == *vault_info.key)?;

    // Get new ORE:stORE ratio.
    let ratio = if stake.balance == 0 || store_mint.supply() == 0 {
        Numeric::from_u64(1)
    } else {
        Numeric::from_fraction(stake.balance, store_mint.supply())
    };

    // Burn stORE tokens.
    let amount = sender_store.amount().min(amount);
    burn(
        sender_store_info,
        store_mint_info,
        signer_info,
        token_program,
        amount,
    )?;

    // Withdraw ORE tokens from stake account.
    let redeemable_amount = (Numeric::from_u64(amount) * ratio).to_u64();
    invoke_signed(
        &ore_api::sdk::withdraw(*vault_info.key, redeemable_amount),
        &[
            vault_info.clone(),
            ore_mint_info.clone(),
            vault_tokens_info.clone(),
            stake_info.clone(),
            stake_tokens_info.clone(),
            treasury_info.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
        &ore_lst_api::ID,
        &[VAULT],
    )?;

    // Transfer ORE tokens to redeemer.
    transfer_signed(
        vault_info,
        vault_tokens_info,
        sender_ore_info,
        token_program,
        redeemable_amount,
        &[VAULT],
    )?;

    Ok(())
}
