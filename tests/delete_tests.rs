use borsh::BorshSerialize;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    signer::Signer,
    transaction::Transaction,
    transport::TransportError,
};
use solana_system_interface::program;
use soldb_program::{
    accounts::{SolTable, SolValue},
    id as program_id,
    instructions::{Delete, SolDbIntructions},
};

use crate::utils::setup;

mod utils;

#[tokio::test]
async fn test_delete() -> Result<(), TransportError> {
    let (banks_client, payer, last_blockhash) = setup().await?;

    let name = "Test".to_string();
    let table = SolTable { name: name.clone() };
    let (pda_table_pubkey, table_bump) =
        utils::init_table(&banks_client, &payer, last_blockhash, &table).await?;

    let key: Vec<u8> = "k-0".into();
    let value: Vec<u8> = "v-0".into();
    let sol_value = SolValue { val: value.clone() };
    let (pda_val_pubkey, value_bump) = utils::insert(
        &banks_client,
        &payer,
        last_blockhash,
        &pda_table_pubkey,
        key.into(),
        &sol_value,
    )
    .await?;

    let delete = Delete {
        table: name.clone(),
        table_bump,
        key: "k-0".into(),
        key_bump: value_bump,
    };

    let instr = SolDbIntructions::Delete(delete);

    let mut ix_data = Vec::new();
    instr.serialize(&mut ix_data).unwrap();

    let ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_table_pubkey, false),
            AccountMeta::new(pda_val_pubkey, false),
            AccountMeta::new_readonly(program::ID, false),
        ],
        data: ix_data,
    };

    let txn = Transaction::new_signed_with_payer(
        &[ix.clone()],
        Some(&payer.pubkey()),
        &[&payer],
        last_blockhash,
    );

    let tx_fee = banks_client
        .get_fee_for_message(txn.message().clone())
        .await
        .unwrap()
        .unwrap();

    let owner_before = banks_client
        .get_account(payer.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;

    let val_before_acc = banks_client
        .get_account(pda_val_pubkey)
        .await
        .unwrap()
        .unwrap();

    banks_client.process_transaction_with_metadata(txn).await?;

    let owner_after = banks_client
        .get_account(payer.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;

    let old_val_lamports = val_before_acc.lamports;

    let expected_owner_after = owner_before - tx_fee + old_val_lamports;
    assert_eq!(
        owner_after, expected_owner_after,
        "El pagador debería recuperar los lamports de la cuenta cerrada menos el fee"
    );

    let val_after_acc = banks_client.get_account(pda_val_pubkey).await.unwrap();
    assert!(
        val_after_acc.is_none(),
        "La cuenta del valor debería estar cerrada tras Delete"
    );

    Ok(())
}
