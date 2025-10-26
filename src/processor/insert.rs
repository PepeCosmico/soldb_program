use std::io::Read;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, account_info::next_account_info, entrypoint::ProgramResult, msg,
    program::invoke, program::invoke_signed, program_error::ProgramError, pubkey::Pubkey,
    rent::Rent, sysvar::Sysvar,
};
use solana_system_interface::instruction;

use crate::{
    accounts::{SolTable, SolValue},
    instructions::Insert,
};

pub fn process_insert<'o, 't, 's>(
    insert: Insert,
    accounts: &[AccountInfo],
    program_id: &Pubkey,
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner_info = next_account_info(account_iter)?;
    let table_info = next_account_info(account_iter)?;
    let pda_info = next_account_info(account_iter)?;
    let sys_prog = next_account_info(account_iter)?;

    let mut sol_table = SolTable::try_from_slice(&table_info.data.borrow()).map_err(|_| {
        msg!("Second Account is not a SolTable Account");
        ProgramError::InvalidAccountData
    })?;

    let sol_value = SolValue {
        val: insert.payload.clone(),
    };
    let mut serialized = Vec::new();
    sol_value.serialize(&mut serialized)?;
    let space = serialized.len() as u64;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space as usize);

    let seeds = &[
        &insert.key.as_ref(),
        table_info.key.as_ref(),
        owner_info.key.as_ref(),
        &[insert.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let ix = instruction::create_account(owner_info.key, pda_info.key, lamports, space, program_id);
    invoke_signed(&ix, accounts, signer_seeds)?;

    sol_value.serialize(&mut &mut pda_info.data.borrow_mut()[..])?;

    sol_table.keys.push(insert.key);

    let mut tbuf = Vec::new();
    sol_table.serialize(&mut tbuf)?;
    let new_len = tbuf.len();

    let current_len = table_info.data_len();

    if new_len != current_len {
        let min_balance_new = rent.minimum_balance(new_len);
        let table_lamports_now = **table_info.lamports.borrow();

        if new_len > current_len {
            if table_lamports_now < min_balance_new {
                let needed = min_balance_new - table_lamports_now;
                let transfer_ix = instruction::transfer(owner_info.key, table_info.key, needed);
                let transfer_accounts = [owner_info.clone(), table_info.clone(), sys_prog.clone()];
                invoke(&transfer_ix, &transfer_accounts)?;
            }

            table_info.resize(new_len)?;
        } else {
            table_info.resize(new_len)?;

            let after = **table_info.lamports.borrow();
            if after > min_balance_new {
                let excess = after - min_balance_new;
                **table_info.lamports.borrow_mut() -= excess;
                **owner_info.lamports.borrow_mut() += excess;
            }
        }

        tbuf.as_slice()
            .read_exact(&mut &mut table_info.data.borrow_mut()[..new_len])
            .map_err(|_| ProgramError::AccountDataTooSmall)?;
    } else {
        let data = &mut table_info.data.borrow_mut()[..new_len];
        data.copy_from_slice(&tbuf);
    }

    Ok(())
}
