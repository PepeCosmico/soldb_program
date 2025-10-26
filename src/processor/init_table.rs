use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{accounts::SolTable, instructions::InitTable, utils::create_account_with_data};

pub fn process_init_table(
    init_table: InitTable,
    owner_info: &AccountInfo,
    pda_info: &AccountInfo,
    accounts: &[AccountInfo],
    program_id: &Pubkey,
) -> ProgramResult {
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
