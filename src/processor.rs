use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{AccountInfo, next_account_info},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};
use solana_program_error::ToStr;
use solana_system_interface::instruction;

use crate::{
    accounts::{SolTable, SolValue},
    error::SolDbError,
    instructions::{Delete, InitTable, Insert, Put, SolDbIntructions},
};
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = SolDbIntructions::unpack(instruction_data)?;

    match instruction {
        SolDbIntructions::InitTable(init_table) => {
            process_init_table(init_table, program_id, accounts)?;
        }
        SolDbIntructions::Insert(insert) => {
            process_insert(insert, program_id, accounts)?;
        }
        SolDbIntructions::Put(put) => {
            process_put(put, program_id, accounts)?;
        }
        SolDbIntructions::Delete(delete) => {
            process_delete(delete, program_id, accounts)?;
        }
    };

    Ok(())
}

fn process_init_table(
    init_table: InitTable,
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner_info = next_account_info(account_iter)?;
    let pda_info = next_account_info(account_iter)?;
    let sys_prog = next_account_info(account_iter)?;

    let (expected_pda, expected_bump) = Pubkey::find_program_address(
        &[&init_table.name.as_ref(), owner_info.key.as_ref()],
        program_id,
    );

    if pda_info.key != &expected_pda || init_table.bump != expected_bump {
        msg!("PDA mismatch");
        return Err(ProgramError::InvalidSeeds);
    }

    let sol_table = SolTable {
        name: init_table.name.clone(),
    };
    let mut serialized = Vec::new();
    sol_table.serialize(&mut serialized)?;
    let space = serialized.len() as u64;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space as usize);

    let seeds = &[
        init_table.name.as_ref(),
        owner_info.key.as_ref(),
        &[init_table.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let ix = instruction::create_account(owner_info.key, pda_info.key, lamports, space, program_id);
    invoke_signed(
        &ix,
        &[owner_info.clone(), pda_info.clone(), sys_prog.clone()],
        signer_seeds,
    )?;

    let sol_table = SolTable {
        name: init_table.name,
    };

    sol_table.serialize(&mut &mut pda_info.data.borrow_mut()[..])?;

    Ok(())
}

fn process_insert(insert: Insert, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let table_info = next_account_info(account_iter)?;
    let pda_info = next_account_info(account_iter)?;
    let owner_info = next_account_info(account_iter)?;
    let sys_prog = next_account_info(account_iter)?;

    let (expected_pda, expected_bump) = Pubkey::find_program_address(
        &[
            &insert.key,
            table_info.key.as_ref(),
            owner_info.key.as_ref(),
        ],
        program_id,
    );

    let _ = SolTable::try_from_slice(&table_info.data.borrow()).map_err(|_| {
        msg!("Second Account is not a SolTable Account");
        ProgramError::InvalidAccountData
    })?;

    if pda_info.key != &expected_pda || insert.bump != expected_bump {
        msg!("PDA mismatch");
        return Err(ProgramError::InvalidSeeds);
    }

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
    invoke_signed(
        &ix,
        &[owner_info.clone(), pda_info.clone(), sys_prog.clone()],
        signer_seeds,
    )?;

    sol_value.serialize(&mut &mut pda_info.data.borrow_mut()[..])?;

    Ok(())
}

fn process_put(put: Put, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner_info = next_account_info(account_iter)?;
    let table_info = next_account_info(account_iter)?;
    let val_info = next_account_info(account_iter)?;
    let sys_prog = next_account_info(account_iter)?;

    require_keys_eq!(table_info.owner, program_id, SolDbError::WrongOwner);
    require_keys_eq!(val_info.owner, program_id, SolDbError::WrongOwner);

    let (expected_table_pda, expected_table_bump) = Pubkey::find_program_address(
        &[&put.table.as_bytes(), owner_info.key.as_ref()],
        program_id,
    );

    require!(
        table_info.key == &expected_table_pda && put.table_bump == expected_table_bump,
        SolDbError::PdaMismatch
    );

    let _ =
        SolTable::try_from_slice(&table_info.data.borrow()).map_err(|_| SolDbError::NotTable)?;

    let (expected_val_pda, expected_val_bump) = Pubkey::find_program_address(
        &[
            &put.key,
            &table_info.key.to_bytes(),
            owner_info.key.as_ref(),
        ],
        program_id,
    );

    require!(
        val_info.key == &expected_val_pda && put.key_bump == expected_val_bump,
        SolDbError::PdaMismatch
    );

    let sol_value = SolValue {
        val: put.payload.clone(),
    };

    let mut serialized = Vec::new();
    sol_value.serialize(&mut serialized)?;
    let new_space = serialized.len() as u64;
    let old_space = val_info.data_len() as u64;

    if new_space > old_space {
        const MAX_INCREASE: u64 = 10 * 1024;
        let inc = new_space - old_space;
        if inc > MAX_INCREASE {
            msg!(
                "Requested increase: {}, max per transaction: {}",
                inc,
                MAX_INCREASE
            );
            return Err(SolDbError::GrowthTooLarge.into());
        }
    }

    let rent = Rent::get()?;
    let new_min = rent.minimum_balance(new_space as usize);

    if new_min > val_info.lamports() {
        let need = new_min.saturating_sub(val_info.lamports());
        invoke(
            &instruction::transfer(owner_info.key, val_info.key, need),
            &[owner_info.clone(), val_info.clone(), sys_prog.clone()],
        )?;
    }

    val_info.resize(new_space as usize)?;

    if new_space < old_space {
        let after_min = rent.minimum_balance(new_space as usize);
        let cur = val_info.lamports();
        if cur > after_min {
            let refund = cur - after_min;
            **val_info.try_borrow_mut_lamports()? -= refund;
            **owner_info.try_borrow_mut_lamports()? += refund;
        }
    }

    sol_value.serialize(&mut &mut val_info.data.borrow_mut()[..])?;

    Ok(())
}

fn process_delete(delete: Delete, program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let account_iter = &mut accounts.iter();
    let owner_info = next_account_info(account_iter)?;
    let table_info = next_account_info(account_iter)?;
    let val_info = next_account_info(account_iter)?;
    let _sys_prog = next_account_info(account_iter)?;

    require_keys_eq!(table_info.owner, program_id, SolDbError::WrongOwner);
    require_keys_eq!(val_info.owner, program_id, SolDbError::WrongOwner);

    let (expected_table_pda, expected_table_bump) = Pubkey::find_program_address(
        &[&delete.table.as_bytes(), owner_info.key.as_ref()],
        program_id,
    );

    require!(
        table_info.key == &expected_table_pda && delete.table_bump == expected_table_bump,
        SolDbError::PdaMismatch
    );

    let _ =
        SolTable::try_from_slice(&table_info.data.borrow()).map_err(|_| SolDbError::NotTable)?;

    let (expected_val_pda, expected_val_bump) = Pubkey::find_program_address(
        &[
            &delete.key,
            &table_info.key.to_bytes(),
            owner_info.key.as_ref(),
        ],
        program_id,
    );

    require!(
        val_info.key == &expected_val_pda && delete.key_bump == expected_val_bump,
        SolDbError::PdaMismatch
    );

    **owner_info.lamports.borrow_mut() += **val_info.lamports.borrow();
    **val_info.lamports.borrow_mut() = 0;

    val_info.data.borrow_mut().fill(0);

    Ok(())
}
