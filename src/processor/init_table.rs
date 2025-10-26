use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
};

use crate::{accounts::SolTable, instructions::InitTable, utils::create_account_with_data};

pub fn process_init_table(
    init_table: InitTable,
    accounts: &[AccountInfo],
    program_id: &Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner_info = next_account_info(account_iter)?;
    let pda_info = next_account_info(account_iter)?;
    let _sys_prog = next_account_info(account_iter)?;

    let sol_table = SolTable::new(init_table.name.clone());

    let seeds = &[
        init_table.name.as_ref(),
        owner_info.key.as_ref(),
        &[init_table.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    create_account_with_data(
        owner_info,
        pda_info,
        sol_table,
        accounts,
        signer_seeds,
        program_id,
    )?;

    Ok(())
}
