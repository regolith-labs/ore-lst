mod compound;
mod initialize;
mod unwrap;
mod wrap;

use compound::*;
use initialize::*;
use unwrap::*;
use wrap::*;

use ore_lst_api::instruction::*;
use solana_security_txt::security_txt;
use steel::*;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (ix, data) = parse_instruction(&ore_lst_api::ID, program_id, data)?;

    match ix {
        // User
        OreLstInstruction::Compound => process_compound(accounts, data)?,
        OreLstInstruction::Wrap => process_wrap(accounts, data)?,
        OreLstInstruction::Unwrap => process_unwrap(accounts, data)?,

        // Admin
        OreLstInstruction::Initialize => process_initialize(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);

security_txt! {
    name: "ORE LST",
    project_url: "https://ore.supply",
    contacts: "email:hardhatchad@gmail.com,discord:hardhatchad",
    policy: "https://github.com/regolith-labs/ore-lst/blob/master/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/regolith-labs/ore-lst"
}
