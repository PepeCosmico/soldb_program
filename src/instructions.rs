use crate::error::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub enum SolDbIntructions {
    /// Initializes a new PDA account for a table.
    ///
    /// Expects the following accounts:
    /// 1. `[signer]` Owner of the table.
    /// 2. `[writable]` PDA to create for the table.
    /// 3. `[]` System program.
    InitTable(InitTable),

    /// Inserts a new key-value pair under an existing table.
    ///
    /// The key-value data is stored in a dedicated PDA derived from
    /// the table name and key bytes.
    ///
    /// Expects the following accounts:
    /// 1. `[]` Table account (must match PDA derived from table name and owner).
    /// 2. `[writable]` Key-value PDA to be created.
    /// 3. `[signer]` Payer for account creation.
    /// 4. `[]` System program.
    Insert(Insert),

    /// Updates an existing key-value pair under a table.
    ///
    /// Expects the following accounts:
    /// 1. `[writable]` Key-value PDA to overwrite.
    Put(Put),

    /// Deletes a key-value pair by closing its PDA.
    ///
    /// Expects the following accounts:
    /// 1. `[writable]` Key-value PDA to close.
    /// 2. `[writable]` Recipient of lamports from the closed account.
    Delete(Delete),
}

impl SolDbIntructions {
    pub fn unpack(input: &[u8]) -> Result<Self> {
        let (&variant, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        match variant {
            0 => {
                let create_table = InitTable::try_from_slice(rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::InitTable(create_table))
            }
            1 => {
                let insert = Insert::try_from_slice(rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::Insert(insert))
            }
            2 => {
                let put =
                    Put::try_from_slice(rest).map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::Put(put))
            }
            3 => {
                let delete = Delete::try_from_slice(rest)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                Ok(Self::Delete(delete))
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct InitTable {
    pub name: String,
    pub bump: u8,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Insert {
    pub key: Vec<u8>,
    pub payload: Vec<u8>,
    pub bump: u8,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Put {
    pub table: String,
    pub table_bump: u8,
    pub key: Vec<u8>,
    pub key_bump: u8,
    pub payload: Vec<u8>,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct Delete {
    pub table: String,
    pub table_bump: u8,
    pub key: Vec<u8>,
    pub key_bump: u8,
}
