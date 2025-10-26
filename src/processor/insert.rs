use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, msg, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};
use solana_system_interface::instruction;

use crate::{
    accounts::{SolTable, SolValue},
    instructions::Insert,
};

pub fn process_insert(
    insert: Insert,
    owner_info: &AccountInfo,
    table_info: &AccountInfo,
    pda_info: &AccountInfo,
    accounts: &[AccountInfo],
    program_id: &Pubkey,
) -> ProgramResult {
    let _ = SolTable::try_from_slice(&table_info.data.borrow()).map_err(|_| {
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
        &insert.key,
        table_info.key.as_ref(),
        owner_info.key.as_ref(),
        &[insert.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let ix = instruction::create_account(owner_info.key, pda_info.key, lamports, space, program_id);
    invoke_signed(&ix, accounts, signer_seeds)?;

    sol_value.serialize(&mut &mut pda_info.data.borrow_mut()[..])?;

    Ok(())
}
