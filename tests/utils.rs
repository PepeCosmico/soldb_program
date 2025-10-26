#![allow(dead_code)]

use borsh::BorshSerialize;
use solana_program_test::*;
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
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
    let mut program_test = ProgramTest::new(
        "soldb_program",
        pid,
        processor!(soldb_program::processor::process_instruction),
    );

    program_test.prefer_bpf(false);

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
        AccountMeta::new_readonly(payer.pubkey(), true),
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
    key: String,
    sol_value: &SolValue,
) -> Result<(Pubkey, u8), TransportError> {
    let program_id = soldb_program::id();

    let (pda_pubkey, bump) = Pubkey::find_program_address(
        &[key.as_ref(), &table.to_bytes(), payer.pubkey().as_ref()],
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
        AccountMeta::new_readonly(payer.pubkey(), true),
        AccountMeta::new(table.clone(), false),
        AccountMeta::new(pda_pubkey, false),
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

pub async fn send_ix(
    banks_client: &mut BanksClient,
    payer: &Keypair,
    recent_blockhash: solana_sdk::hash::Hash,
    ix: Instruction,
    extra_signers: &[&Keypair],
) -> Result<(), TransportError> {
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer.pubkey()));
    let mut signers: Vec<&Keypair> = vec![payer];
    signers.extend_from_slice(extra_signers);
    tx.sign(&signers, recent_blockhash);
    banks_client
        .process_transaction(tx)
        .await
        .map_err(to_transport_error)
}

fn to_transport_error(e: BanksClientError) -> TransportError {
    match e {
        BanksClientError::TransactionError(tx_err) => TransportError::TransactionError(tx_err),
        other => TransportError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("BanksClientError: {other:?}"),
        )),
    }
}
