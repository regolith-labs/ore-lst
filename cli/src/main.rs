use ore_lst_api::consts::STORE_MINT_ADDRESS;
use ore_mint_api::consts::TOKEN_DECIMALS;
use ore_stake_api::state::{Stake, Treasury, Vesting};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};
use spl_token::{amount_to_ui_amount, ui_amount_to_amount};
use steel::{AccountDeserialize, Clock, Numeric};

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
    // Fetch stake account..
    let vault_address = ore_lst_api::state::vault_pda().0;
    let stake_address = ore_stake_api::state::stake_pda(vault_address).0;
    let stake_data = rpc.get_account_data(&stake_address).await.unwrap();
    let mut stake = *Stake::try_from_bytes(&stake_data).unwrap();

    // Fetch treasury account.
    let treasury_address = ore_stake_api::state::treasury_pda().0;
    let treasury_data = rpc.get_account_data(&treasury_address).await.unwrap();
    let mut treasury = *Treasury::try_from_bytes(&treasury_data).unwrap();

    // Fetch vesting account.
    let vesting_address = ore_stake_api::state::vesting_pda().0;
    let vesting_data = rpc.get_account_data(&vesting_address).await.unwrap();
    let mut vesting = *Vesting::try_from_bytes(&vesting_data).unwrap();

    // Get clock
    let clock = get_clock(rpc).await.unwrap();

    // Update stake rewards for total deposits.
    stake.update_rewards(&clock, &mut treasury, &mut vesting);
    let compounded_balance = stake.balance + stake.rewards;

    // Get stORE supply.
    let store_mint_supply = rpc.get_token_supply(&STORE_MINT_ADDRESS).await.unwrap();
    let store_mint_supply_u64 = store_mint_supply.amount.parse::<u64>().unwrap();

    // Get new ORE:stORE ratio.
    let ratio = if store_mint_supply_u64 == 0 || compounded_balance == 0 {
        Numeric::from_u64(1)
    } else {
        Numeric::from_fraction(compounded_balance, store_mint_supply_u64)
    };

    // Print results.
    println!(
        "Stake: {} ORE",
        amount_to_ui_amount(compounded_balance, TOKEN_DECIMALS)
    );
    println!("Supply: {} stORE", store_mint_supply.ui_amount_string);
    println!("--------------------------------");
    println!("1 stORE = {:.11} ORE", ratio.to_i80f48().to_num::<f64>());
    Ok(())
}

async fn get_clock(rpc: &RpcClient) -> Result<Clock, anyhow::Error> {
    let data = rpc.get_account_data(&solana_sdk::sysvar::clock::ID).await?;
    let clock = bincode::deserialize::<Clock>(&data)?;
    Ok(clock)
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
