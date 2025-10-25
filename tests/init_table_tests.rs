mod utils;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transport::TransportError,
};
use solana_system_interface::program;

use soldb_program::{
    accounts::SolTable,
    instructions::{InitTable, SolDbIntructions},
};

use crate::utils::send_ix;

#[tokio::test]
async fn test_init_table_ok() -> Result<(), TransportError> {
    let (banks_client, payer, last_blockhash) = utils::setup().await?;

    let program_id = soldb_program::id();

    let name = "TestInit".to_string();
    let table = SolTable {
        name: name.clone(),
        keys: Vec::new(),
    };
    let (pda_pubkey, _bump) =
        utils::init_table(&banks_client, &payer, last_blockhash, &table).await?;

    let mut serialized = Vec::new();
    table.serialize(&mut serialized)?;

    let expected_space = serialized.len();
    let rent = banks_client
        .get_rent()
        .await
        .map_err(|_| TransportError::Custom("Error getting Rent".to_string()))?;
    let min_lamports = rent.minimum_balance(expected_space);

    let maybe_account = banks_client.get_account(pda_pubkey).await?;
    assert!(maybe_account.is_some(), "PDA account was not created");

    let account = maybe_account.unwrap();
    assert_eq!(
        account.owner, program_id,
        "PDA account is not owned by the program"
    );
    assert!(
        account.lamports >= min_lamports,
        "PDA account has insufficient lamports ({} < {})",
        account.lamports,
        min_lamports
    );
    assert_eq!(
        account.data.len(),
        expected_space,
        "PDA account data length mismatch"
    );

    let sol_table = SolTable::deserialize(&mut account.data.as_slice()).unwrap();
    assert_eq!(sol_table.name, name);

    Ok(())
}

#[tokio::test]
async fn test_init_table_pda_mismatch_name() -> Result<(), TransportError> {
    let (mut banks_client, payer, recent_blockhash) = utils::setup().await?;
    let program_id = soldb_program::id();

    // Derivamos PDA con "Test"
    let correct_name = "TestInitPdaMismatch".to_string();
    let wrong_name = "Wrong".to_string();
    let (pda_pubkey, bump) = Pubkey::find_program_address(
        &[correct_name.as_bytes(), payer.pubkey().as_ref()],
        &program_id,
    );

    // Pero enviamos instrucción con "Wrong"
    let init = InitTable {
        name: wrong_name.clone(),
        bump,
    };
    let mut data = Vec::new();
    SolDbIntructions::InitTable(init)
        .serialize(&mut data)
        .unwrap();

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(payer.pubkey(), true), // owner (signer)
            AccountMeta::new(pda_pubkey, false),             // pda (writable)
            AccountMeta::new_readonly(program::ID, false),
        ],
        data,
    };

    let err = send_ix(&mut banks_client, &payer, recent_blockhash, ix, &[])
        .await
        .unwrap_err();

    // Debe ser InvalidSeeds
    let TransportError::TransactionError(tx_err) = err else {
        panic!("Unexpected error type");
    };
    match tx_err {
        solana_sdk::transaction::TransactionError::InstructionError(_, ie) => {
            assert_eq!(ie, solana_sdk::instruction::InstructionError::InvalidSeeds);
        }
        _ => panic!("Expected InstructionError::InvalidSeeds, got {tx_err:?}"),
    }

    Ok(())
}

#[tokio::test]
async fn test_init_table_wrong_bump() -> Result<(), TransportError> {
    let (mut banks_client, payer, recent_blockhash) = utils::setup().await?;
    let program_id = soldb_program::id();

    let name = "TestInitWrongBump".to_string();
    let (pda_pubkey, correct_bump) =
        Pubkey::find_program_address(&[name.as_bytes(), payer.pubkey().as_ref()], &program_id);

    // Usamos un bump incorrecto (simplemente +1 mod 256)
    let wrong_bump = correct_bump.wrapping_add(1);

    let init = InitTable {
        name: name.clone(),
        bump: wrong_bump,
    };
    let mut data = Vec::new();
    SolDbIntructions::InitTable(init)
        .serialize(&mut data)
        .unwrap();

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(payer.pubkey(), true),
            AccountMeta::new(pda_pubkey, false),
            AccountMeta::new_readonly(program::ID, false),
        ],
        data,
    };

    let err = send_ix(&mut banks_client, &payer, recent_blockhash, ix, &[])
        .await
        .unwrap_err();

    let TransportError::TransactionError(tx_err) = err else {
        panic!("Unexpected error type");
    };
    match tx_err {
        solana_sdk::transaction::TransactionError::InstructionError(_, ie) => {
            assert_eq!(ie, solana_sdk::instruction::InstructionError::InvalidSeeds);
        }
        _ => panic!("Expected InstructionError::InvalidSeeds, got {tx_err:?}"),
    }

    Ok(())
}

#[tokio::test]
async fn test_init_table_owner_not_signer() -> Result<(), TransportError> {
    let (mut banks_client, payer, recent_blockhash) = utils::setup().await?;
    let program_id = soldb_program::id();

    // Usamos un "owner" que NO va a firmar la transacción
    let owner = Keypair::new();
    let name = "TestInitOwnerNotSigner".to_string();
    let (pda_pubkey, bump) =
        Pubkey::find_program_address(&[name.as_bytes(), owner.pubkey().as_ref()], &program_id);

    let init = InitTable {
        name: name.clone(),
        bump,
    };
    let mut data = Vec::new();
    SolDbIntructions::InitTable(init)
        .serialize(&mut data)
        .unwrap();

    // OJO: marcamos owner como NO signer (false) y no lo añadimos a signers
    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(owner.pubkey(), false), // ❌ no signer
            AccountMeta::new(pda_pubkey, false),
            AccountMeta::new_readonly(program::ID, false),
        ],
        data,
    };

    let err = send_ix(&mut banks_client, &payer, recent_blockhash, ix, &[])
        .await
        .unwrap_err();

    // Debe ser MissingRequiredSignature a nivel de transacción
    let TransportError::TransactionError(tx_err) = err else {
        panic!("Unexpected error type");
    };
    assert_eq!(
        tx_err,
        solana_sdk::transaction::TransactionError::InstructionError(
            0,
            solana_sdk::instruction::InstructionError::MissingRequiredSignature
        )
    );

    Ok(())
}

#[tokio::test]
async fn test_init_table_wrong_system_program() -> Result<(), TransportError> {
    let (mut banks_client, payer, recent_blockhash) = utils::setup().await?;
    let program_id = soldb_program::id();

    let name = "TestInitWrongSysProgram".to_string();
    let (pda_pubkey, bump) =
        Pubkey::find_program_address(&[name.as_bytes(), payer.pubkey().as_ref()], &program_id);

    let fake_sys = Keypair::new();

    let init = InitTable {
        name: name.clone(),
        bump,
    };
    let mut data = Vec::new();
    SolDbIntructions::InitTable(init)
        .serialize(&mut data)
        .unwrap();

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(payer.pubkey(), true),
            AccountMeta::new(pda_pubkey, false),
            AccountMeta::new_readonly(fake_sys.pubkey(), false),
        ],
        data,
    };

    let err = send_ix(&mut banks_client, &payer, recent_blockhash, ix, &[])
        .await
        .unwrap_err();

    let TransportError::TransactionError(tx_err) = err else {
        panic!("Unexpected error type");
    };
    match tx_err {
        solana_sdk::transaction::TransactionError::InstructionError(_, ie) => {
            assert_eq!(
                ie,
                solana_sdk::instruction::InstructionError::IncorrectProgramId
            );
        }
        _ => panic!("Expected InstructionError::IncorrectProgramId, got {tx_err:?}"),
    }

    Ok(())
}

#[tokio::test]
async fn test_init_table_pda_not_writer() -> Result<(), TransportError> {
    let (mut banks_client, payer, recent_blockhash) = utils::setup().await?;
    let program_id = soldb_program::id();

    let owner = Keypair::new();
    let name = "TestInitPdaNotWritable".to_string();
    let (pda_pubkey, bump) =
        Pubkey::find_program_address(&[name.as_bytes(), owner.pubkey().as_ref()], &program_id);

    let init = InitTable {
        name: name.clone(),
        bump,
    };
    let mut data = Vec::new();
    SolDbIntructions::InitTable(init)
        .serialize(&mut data)
        .unwrap();

    let ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(payer.pubkey(), true),
            AccountMeta::new_readonly(pda_pubkey, false),
            AccountMeta::new_readonly(program::ID, false),
        ],
        data,
    };

    let err = send_ix(&mut banks_client, &payer, recent_blockhash, ix, &[])
        .await
        .unwrap_err();

    let TransportError::TransactionError(tx_err) = err else {
        panic!("Unexpected error type");
    };
    assert_eq!(
        tx_err,
        solana_sdk::transaction::TransactionError::InstructionError(
            0,
            solana_sdk::instruction::InstructionError::Custom(5)
        )
    );

    Ok(())
}
