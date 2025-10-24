use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed, pubkey::Pubkey,
    rent::Rent, sysvar::Sysvar,
};
use solana_system_interface::instruction;

pub fn create_account(
    owner_info: &AccountInfo,
    pda_info: &AccountInfo,
    space: u64,
    accounts: &[AccountInfo],
    signer_seeds: &[&[&[u8]]],
    program_id: &Pubkey,
) -> ProgramResult {
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space as usize);

    let ix = instruction::create_account(owner_info.key, pda_info.key, lamports, space, program_id);
    invoke_signed(&ix, accounts, signer_seeds)?;

    Ok(())
}

pub fn create_account_with_data<T>(
    owner_info: &AccountInfo,
    pda_info: &AccountInfo,
    data: T,
    accounts: &[AccountInfo],
    signer_seeds: &[&[&[u8]]],
    program_id: &Pubkey,
) -> ProgramResult
where
    T: BorshDeserialize + BorshSerialize,
{
    let mut serialized = Vec::new();
    data.serialize(&mut serialized)?;
    let space = serialized.len() as u64;

    create_account(
        owner_info,
        pda_info,
        space,
        accounts,
        signer_seeds,
        program_id,
    )?;

    data.serialize(&mut &mut pda_info.data.borrow_mut()[..])?;

    Ok(())
}
