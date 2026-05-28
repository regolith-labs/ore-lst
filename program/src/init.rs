use ore_lst_api::{
    consts::{STORE_MINT_ADDRESS, VAULT},
    state::{vault_pda, Vault},
};
use ore_mint_api::consts::MINT_ADDRESS;
use ore_stake_api::state::Stake;
use steel::*;

/// Initialize the program.
pub fn process_init(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, ore_mint_info, store_mint_info, metadata_info, stake_info, stake_tokens_info, treasury_info, vault_info, vault_tokens_info, vesting_info, system_program, token_program, associated_token_program, metadata_program, ore_stake_program, rent_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    ore_mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    store_mint_info
        .has_address(&STORE_MINT_ADDRESS)?
        .as_mint()?;
    vault_info.has_address(&vault_pda().0)?.is_writable()?;
    vault_tokens_info.is_writable()?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;
    metadata_program.is_program(&mpl_token_metadata::ID)?;
    ore_stake_program.is_program(&ore_stake_api::ID)?;
    rent_sysvar.is_sysvar(&sysvar::rent::ID)?;

    // Initialize mint metadata
    let vault_bump = vault_pda().1;
    mpl_token_metadata::instructions::CreateMetadataAccountV3Cpi {
        __program: metadata_program,
        metadata: metadata_info,
        mint: store_mint_info,
        mint_authority: vault_info,
        payer: signer_info,
        update_authority: (vault_info, true),
        system_program,
        rent: Some(rent_sysvar),
        __args: mpl_token_metadata::instructions::CreateMetadataAccountV3InstructionArgs {
            data: mpl_token_metadata::types::DataV2 {
                name: "Staked ORE".to_string(),
                symbol: "stORE".to_string(),
                uri: "https://ore.supply/assets/metadata-lst.json".to_string(),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            is_mutable: true,
            collection_details: None,
        },
    }
    .invoke_signed(&[&[VAULT, &[vault_bump]]])?;

    // Open vault.
    if vault_info.data_is_empty() {
        create_program_account::<Vault>(
            vault_info,
            system_program,
            signer_info,
            &ore_lst_api::ID,
            &[VAULT],
        )?;
    } else {
        vault_info.as_account::<Vault>(&ore_lst_api::ID)?;
    }

    // Open vault token account.
    if vault_tokens_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            vault_info,
            vault_tokens_info,
            ore_mint_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        vault_tokens_info.as_associated_token_account(vault_info.key, &MINT_ADDRESS)?;
    }

    // Create stake account.
    if stake_info.data_is_empty() {
        invoke_signed(
            &ore_stake_api::sdk::deposit(*vault_info.key, *signer_info.key, 0, 0, 0),
            &[
                vault_info.clone(),
                signer_info.clone(),
                ore_mint_info.clone(),
                vault_tokens_info.clone(),
                stake_info.clone(),
                stake_tokens_info.clone(),
                treasury_info.clone(),
                vesting_info.clone(),
                system_program.clone(),
                token_program.clone(),
                associated_token_program.clone(),
                ore_stake_program.clone(),
            ],
            &ore_lst_api::ID,
            &[VAULT],
        )?;
    } else {
        stake_info
            .as_account::<Stake>(&ore_stake_api::ID)?
            .assert(|s| s.authority == *vault_info.key)?;
    }

    Ok(())
}
