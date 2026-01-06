use ore_api::{
    consts::MINT_ADDRESS,
    state::{stake_pda, treasury_pda},
};
use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use steel::*;

use crate::{consts::*, instruction::*, state::*};

/// Builds an instruction to initialize the vault and stORE mint.
pub fn init(signer: Pubkey) -> Instruction {
    let vault_address = vault_pda().0;
    let vault_tokens = get_associated_token_address(&vault_address, &MINT_ADDRESS);
    let stake_address = stake_pda(vault_address).0;
    let stake_tokens_address = get_associated_token_address(&stake_address, &MINT_ADDRESS);
    let treasury_address = treasury_pda().0;
    let metadata_address = mpl_token_metadata::accounts::Metadata::find_pda(&STORE_MINT_ADDRESS).0;
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(STORE_MINT_ADDRESS, false),
            AccountMeta::new(metadata_address, false),
            AccountMeta::new(stake_address, false),
            AccountMeta::new(stake_tokens_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new(vault_address, false),
            AccountMeta::new(vault_tokens, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(mpl_token_metadata::ID, false),
            AccountMeta::new_readonly(ore_api::ID, false),
            AccountMeta::new_readonly(sysvar::rent::ID, false),
        ],
        data: Initialize {}.to_bytes(),
    }
}

/// Builds an instruction to wrap ORE into stORE tokens.
///
/// Deposits the specified amount of ORE into the vault and mints stORE to the signer.
pub fn wrap(signer: Pubkey, payer: Pubkey, amount: u64) -> Instruction {
    let sender_ore_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    let sender_store_address = get_associated_token_address(&signer, &STORE_MINT_ADDRESS);
    let ore_mint_address = MINT_ADDRESS;
    let store_mint_address = STORE_MINT_ADDRESS;
    let vault_address = vault_pda().0;
    let vault_tokens = get_associated_token_address(&vault_address, &MINT_ADDRESS);
    let stake_address = stake_pda(vault_address).0;
    let stake_tokens_address = get_associated_token_address(&stake_address, &MINT_ADDRESS);
    let treasury_address = treasury_pda().0;
    let treasury_tokens_address = get_associated_token_address(&treasury_address, &MINT_ADDRESS);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(payer, true),
            AccountMeta::new(sender_ore_address, false),
            AccountMeta::new(sender_store_address, false),
            AccountMeta::new(ore_mint_address, false),
            AccountMeta::new(store_mint_address, false),
            AccountMeta::new(stake_address, false),
            AccountMeta::new(stake_tokens_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new(treasury_tokens_address, false),
            AccountMeta::new(vault_address, false),
            AccountMeta::new(vault_tokens, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(ore_api::ID, false),
        ],
        data: Wrap {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

/// Builds an instruction to unwrap stORE back into ORE tokens.
///
/// Burns the specified amount of stORE and returns the corresponding ORE to the signer.
pub fn unwrap(signer: Pubkey, payer: Pubkey, amount: u64) -> Instruction {
    let sender_ore_address = get_associated_token_address(&signer, &MINT_ADDRESS);
    let sender_store_address = get_associated_token_address(&signer, &STORE_MINT_ADDRESS);
    let ore_mint_address = MINT_ADDRESS;
    let store_mint_address = STORE_MINT_ADDRESS;
    let vault_address = vault_pda().0;
    let vault_tokens = get_associated_token_address(&vault_address, &MINT_ADDRESS);
    let stake_address = stake_pda(vault_address).0;
    let stake_tokens_address = get_associated_token_address(&stake_address, &MINT_ADDRESS);
    let treasury_address = treasury_pda().0;
    let treasury_tokens_address = get_associated_token_address(&treasury_address, &MINT_ADDRESS);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(payer, true),
            AccountMeta::new(sender_ore_address, false),
            AccountMeta::new(sender_store_address, false),
            AccountMeta::new(ore_mint_address, false),
            AccountMeta::new(store_mint_address, false),
            AccountMeta::new(stake_address, false),
            AccountMeta::new(stake_tokens_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new(treasury_tokens_address, false),
            AccountMeta::new(vault_address, false),
            AccountMeta::new(vault_tokens, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(ore_api::ID, false),
        ],
        data: Unwrap {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

/// Builds an instruction to compound staking rewards.
///
/// Claims pending ORE staking rewards and re-stakes them to increase vault value.
pub fn compound(signer: Pubkey) -> Instruction {
    let ore_mint_address = MINT_ADDRESS;
    let treasury_address = treasury_pda().0;
    let treasury_tokens_address = get_associated_token_address(&treasury_address, &MINT_ADDRESS);
    let vault_address = vault_pda().0;
    let vault_tokens = get_associated_token_address(&vault_address, &MINT_ADDRESS);
    let stake_address = stake_pda(vault_address).0;
    let stake_tokens_address = get_associated_token_address(&stake_address, &MINT_ADDRESS);
    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(ore_mint_address, false),
            AccountMeta::new(stake_address, false),
            AccountMeta::new(stake_tokens_address, false),
            AccountMeta::new(treasury_address, false),
            AccountMeta::new(treasury_tokens_address, false),
            AccountMeta::new(vault_address, false),
            AccountMeta::new(vault_tokens, false),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(spl_token::ID, false),
            AccountMeta::new_readonly(spl_associated_token_account::ID, false),
            AccountMeta::new_readonly(ore_api::ID, false),
        ],
        data: Compound {}.to_bytes(),
    }
}
