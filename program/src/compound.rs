use ore_api::{
    consts::MINT_ADDRESS,
    state::{Stake, Treasury},
};
use ore_lst_api::prelude::*;
use steel::*;

/// Compounds yield into vault.
pub fn process_compound(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, ore_mint_info, stake_info, stake_tokens_info, treasury_info, treasury_tokens_info, vault_info, vault_tokens_info, system_program, token_program, associated_token_program, ore_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    ore_mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
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

    Ok(())
}
