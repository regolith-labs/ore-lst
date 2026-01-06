use ore_api::{
    consts::TOKEN_DECIMALS,
    state::{Stake, Treasury},
};
use ore_lst_api::consts::STORE_MINT_ADDRESS;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    client_error::{reqwest::StatusCode, ClientErrorKind},
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_sdk::{
    address_lookup_table::AddressLookupTableAccount,
    compute_budget::ComputeBudgetInstruction,
    message::{v0::Message, VersionedMessage},
    pubkey::Pubkey,
    signature::{read_keypair_file, Signature, Signer},
    transaction::{Transaction, VersionedTransaction},
};
use spl_token::{amount_to_ui_amount, ui_amount_to_amount};
use steel::{AccountDeserialize, Discriminator, Instruction, Numeric};

#[tokio::main]
async fn main() {
    // Read keypair from file
    let payer =
        read_keypair_file(&std::env::var("KEYPAIR").expect("Missing KEYPAIR env var")).unwrap();

    // Build transaction
    let rpc = RpcClient::new(std::env::var("RPC").expect("Missing RPC env var"));
    match std::env::var("COMMAND")
        .expect("Missing COMMAND env var")
        .as_str()
    {
        "initialize" => {
            initialize(&rpc, &payer).await.unwrap();
        }
        "wrap" => {
            wrap(&rpc, &payer).await.unwrap();
        }
        "unwrap" => {
            unwrap(&rpc, &payer).await.unwrap();
        }
        "compound" => {
            compound(&rpc, &payer).await.unwrap();
        }
        "rate" => {
            rate(&rpc).await.unwrap();
        }
        _ => panic!("Invalid command"),
    };
}

async fn initialize(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let ix = ore_lst_api::sdk::init(payer.pubkey());
    submit_transaction(rpc, payer, &[ix]).await.unwrap();
    Ok(())
}

async fn wrap(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let amount_f64 = std::env::var("AMOUNT")
        .expect("Missing AMOUNT env var")
        .parse::<f64>()
        .unwrap();
    let amount_u64 = ui_amount_to_amount(amount_f64, TOKEN_DECIMALS);
    let ix = ore_lst_api::sdk::wrap(payer.pubkey(), payer.pubkey(), amount_u64);
    submit_transaction(rpc, payer, &[ix]).await.unwrap();
    Ok(())
}

async fn unwrap(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let amount_f64 = std::env::var("AMOUNT")
        .expect("Missing AMOUNT env var")
        .parse::<f64>()
        .unwrap();
    let amount_u64 = ui_amount_to_amount(amount_f64, TOKEN_DECIMALS);
    let ix = ore_lst_api::sdk::unwrap(payer.pubkey(), payer.pubkey(), amount_u64);
    submit_transaction(rpc, payer, &[ix]).await.unwrap();
    Ok(())
}

async fn compound(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let ix = ore_lst_api::sdk::compound(payer.pubkey());
    submit_transaction(rpc, payer, &[ix]).await.unwrap();
    rate(rpc).await.unwrap();
    Ok(())
}

async fn rate(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let vault_address = ore_lst_api::state::vault_pda().0;
    let stake_address = ore_api::state::stake_pda(vault_address).0;
    let stake_data = rpc.get_account_data(&stake_address).await.unwrap();
    let mut stake = *Stake::try_from_bytes(&stake_data).unwrap();
    let treasury_address = ore_api::state::treasury_pda().0;
    let treasury_data = rpc.get_account_data(&treasury_address).await.unwrap();
    let treasury = *Treasury::try_from_bytes(&treasury_data).unwrap();
    stake.update_rewards(&treasury);
    let compounded_balance = stake.balance + stake.rewards;
    let store_mint_supply = rpc.get_token_supply(&STORE_MINT_ADDRESS).await.unwrap();
    let store_mint_supply_u64 = store_mint_supply.amount.parse::<u64>().unwrap();

    // Get new ORE:stORE ratio.
    let ratio = if store_mint_supply_u64 > 0 {
        Numeric::from_fraction(compounded_balance, store_mint_supply_u64)
    } else {
        Numeric::from_u64(1)
    };
    println!(
        "Stake: {} ORE",
        amount_to_ui_amount(compounded_balance, TOKEN_DECIMALS)
    );
    println!("Supply: {} stORE", store_mint_supply.ui_amount_string);
    println!("--------------------------------");
    println!("1 stORE = {:.11} ORE", ratio.to_i80f48().to_num::<f64>());
    Ok(())
}

#[allow(dead_code)]
async fn simulate_transaction(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    instructions: &[solana_sdk::instruction::Instruction],
) {
    let blockhash = rpc.get_latest_blockhash().await.unwrap();
    let x = rpc
        .simulate_transaction(&Transaction::new_signed_with_payer(
            instructions,
            Some(&payer.pubkey()),
            &[payer],
            blockhash,
        ))
        .await;
    println!("Simulation result: {:?}", x);
}

#[allow(dead_code)]
async fn simulate_transaction_with_address_lookup_tables(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    instructions: &[solana_sdk::instruction::Instruction],
    address_lookup_table_accounts: Vec<AddressLookupTableAccount>,
) {
    let blockhash = rpc.get_latest_blockhash().await.unwrap();
    let tx = VersionedTransaction {
        signatures: vec![Signature::default()],
        message: VersionedMessage::V0(
            Message::try_compile(
                &payer.pubkey(),
                instructions,
                &address_lookup_table_accounts,
                blockhash,
            )
            .unwrap(),
        ),
    };
    let s = tx.sanitize();
    println!("Sanitize result: {:?}", s);
    s.unwrap();
    let x = rpc.simulate_transaction(&tx).await;
    println!("Simulation result: {:?}", x);
}

#[allow(unused)]
async fn submit_transaction_batches(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    mut ixs: Vec<solana_sdk::instruction::Instruction>,
    batch_size: usize,
) -> Result<(), anyhow::Error> {
    // Batch and submit the instructions.
    while !ixs.is_empty() {
        let batch = ixs
            .drain(..std::cmp::min(batch_size, ixs.len()))
            .collect::<Vec<Instruction>>();
        submit_transaction_no_confirm(rpc, payer, &batch).await?;
    }
    Ok(())
}

#[allow(unused)]
async fn simulate_transaction_batches(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    mut ixs: Vec<solana_sdk::instruction::Instruction>,
    batch_size: usize,
) -> Result<(), anyhow::Error> {
    // Batch and submit the instructions.
    while !ixs.is_empty() {
        let batch = ixs
            .drain(..std::cmp::min(batch_size, ixs.len()))
            .collect::<Vec<Instruction>>();
        simulate_transaction(rpc, payer, &batch).await;
    }
    Ok(())
}

async fn submit_transaction(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    instructions: &[solana_sdk::instruction::Instruction],
) -> Result<solana_sdk::signature::Signature, anyhow::Error> {
    let blockhash = rpc.get_latest_blockhash().await?;
    let mut all_instructions = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
        ComputeBudgetInstruction::set_compute_unit_price(1_000_000),
    ];
    all_instructions.extend_from_slice(instructions);
    let transaction = Transaction::new_signed_with_payer(
        &all_instructions,
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    );

    match rpc.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => {
            println!("Transaction submitted: {:?}", signature);
            Ok(signature)
        }
        Err(e) => {
            println!("Error submitting transaction: {:?}", e);
            Err(e.into())
        }
    }
}

async fn submit_transaction_no_confirm(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    instructions: &[solana_sdk::instruction::Instruction],
) -> Result<solana_sdk::signature::Signature, anyhow::Error> {
    let blockhash = rpc.get_latest_blockhash().await?;
    let mut all_instructions = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
        ComputeBudgetInstruction::set_compute_unit_price(1_000_000),
    ];
    all_instructions.extend_from_slice(instructions);
    let transaction = Transaction::new_signed_with_payer(
        &all_instructions,
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    );

    match rpc.send_transaction(&transaction).await {
        Ok(signature) => {
            println!("Transaction submitted: {:?}", signature);
            Ok(signature)
        }
        Err(e) => {
            println!("Error submitting transaction: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn get_program_accounts<T>(
    client: &RpcClient,
    program_id: Pubkey,
    filters: Vec<RpcFilterType>,
) -> Result<Vec<(Pubkey, T)>, anyhow::Error>
where
    T: AccountDeserialize + Discriminator + Clone,
{
    let mut all_filters = vec![RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
        0,
        &T::discriminator().to_le_bytes(),
    ))];
    all_filters.extend(filters);
    let result = client
        .get_program_accounts_with_config(
            &program_id,
            RpcProgramAccountsConfig {
                filters: Some(all_filters),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await;

    match result {
        Ok(accounts) => {
            let accounts = accounts
                .into_iter()
                .filter_map(|(pubkey, account)| {
                    if let Ok(account) = T::try_from_bytes(&account.data) {
                        Some((pubkey, account.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            Ok(accounts)
        }
        Err(err) => match err.kind {
            ClientErrorKind::Reqwest(err) => {
                if let Some(status_code) = err.status() {
                    if status_code == StatusCode::GONE {
                        panic!(
                                "\n{} Your RPC provider does not support the getProgramAccounts endpoint, needed to execute this command. Please use a different RPC provider.\n",
                                "ERROR"
                            );
                    }
                }
                return Err(anyhow::anyhow!("Failed to get program accounts: {}", err));
            }
            _ => return Err(anyhow::anyhow!("Failed to get program accounts: {}", err)),
        },
    }
}
