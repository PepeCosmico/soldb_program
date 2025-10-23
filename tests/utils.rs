#![allow(dead_code)]

use borsh::BorshSerialize;
use solana_program_test::{BanksClient, ProgramTest};
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
    transport::TransportError,
};
use solana_system_interface::program;
use soldb_program::{
    accounts::{SolTable, SolValue},
    id as program_id,
    instructions::SolDbIntructions,
};

pub async fn setup() -> Result<(BanksClient, Keypair, Hash), TransportError> {
    let pid = Pubkey::new_from_array(program_id().to_bytes());
    let program_test = ProgramTest::new("soldb_program", pid, None);

    Ok(program_test.start().await)
}

pub async fn init_table(
    banks_client: &BanksClient,
    payer: &Keypair,
    last_blockhash: Hash,
    table: &SolTable,
) -> Result<(Pubkey, u8), TransportError> {
    let program_id = soldb_program::id();

    let (pda_pubkey, bump) = Pubkey::find_program_address(
        &[table.name.as_bytes(), payer.pubkey().as_ref()],
        &program_id,
    );

    let instr = SolDbIntructions::InitTable(soldb_program::instructions::InitTable {
        name: table.name.clone(),
        bump,
    });
    let mut ix_data = Vec::new();
    instr.serialize(&mut ix_data).unwrap();

    let accounts = vec![
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new(pda_pubkey, false),
        AccountMeta::new_readonly(program::ID, false),
    ];

    let ix = Instruction {
        program_id: soldb_program::id(),
        accounts,
        data: ix_data,
    };

    let txn =
        Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], last_blockhash);

    banks_client.process_transaction_with_metadata(txn).await?;

    Ok((pda_pubkey, bump))
}

pub async fn insert(
    banks_client: &BanksClient,
    payer: &Keypair,
    last_blockhash: Hash,
    table: &Pubkey,
    key: Vec<u8>,
    sol_value: &SolValue,
) -> Result<(Pubkey, u8), TransportError> {
    let program_id = soldb_program::id();

    let (pda_pubkey, bump) = Pubkey::find_program_address(
        &[&key, &table.to_bytes(), payer.pubkey().as_ref()],
        &program_id,
    );

    let instr = SolDbIntructions::Insert(soldb_program::instructions::Insert {
        key,
        payload: sol_value.val.clone(),
        bump,
    });
    let mut ix_data = Vec::new();
    instr.serialize(&mut ix_data).unwrap();

    let accounts = vec![
        AccountMeta::new_readonly(table.clone(), false),
        AccountMeta::new(pda_pubkey, false),
        AccountMeta::new(payer.pubkey(), true),
        AccountMeta::new_readonly(program::id(), false),
    ];

    let ix = Instruction {
        program_id: soldb_program::id(),
        accounts,
        data: ix_data,
    };

    let txn =
        Transaction::new_signed_with_payer(&[ix], Some(&payer.pubkey()), &[&payer], last_blockhash);

    banks_client.process_transaction_with_metadata(txn).await?;

    Ok((pda_pubkey, bump))
}
