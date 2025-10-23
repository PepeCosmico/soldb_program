mod utils;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program_test::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    rent::Rent,
    signer::Signer,
    transaction::Transaction,
    transport::TransportError,
};
use solana_system_interface::program;
use soldb_program::{
    accounts::{SolTable, SolValue},
    id as program_id,
    instructions::{Put, SolDbIntructions},
};
use utils::setup;

#[tokio::test]
async fn test_put() -> Result<(), TransportError> {
    let (banks_client, payer, last_blockhash) = setup().await?;

    let name = "Test".to_string();
    let table = SolTable { name };
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

    let new_payload: Vec<u8> = "v-10".into();
    let instr = SolDbIntructions::Put(Put {
        table: "Test".to_string(),
        table_bump,
        key: "k-0".into(),
        key_bump: value_bump,
        payload: new_payload.clone(),
    });
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
    let old_len = val_before_acc.data.len();
    let old_min = Rent::default().minimum_balance(old_len);

    banks_client.process_transaction_with_metadata(txn).await?;

    let val_after_acc = banks_client
        .get_account(pda_val_pubkey)
        .await
        .unwrap()
        .unwrap();
    let stored: SolValue = BorshDeserialize::try_from_slice(&val_after_acc.data).unwrap();
    assert_eq!(stored.val, new_payload, "payload must be updated");

    let new_len = val_after_acc.data.len();
    assert!(new_len >= old_len, "should have grown or stayed same");
    let new_min = Rent::default().minimum_balance(new_len);
    assert_eq!(
        val_after_acc.lamports, new_min,
        "value account should be funded exactly to new rent minimum"
    );

    let owner_after = banks_client
        .get_account(payer.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;

    let actual_paid = owner_before.saturating_sub(owner_after);
    let actual_topup = actual_paid.saturating_sub(tx_fee);
    let expected_topup = new_min.saturating_sub(old_min);

    assert_eq!(
        actual_topup, expected_topup,
        "payer should fund exactly the rent delta (net of tx fee)"
    );

    Ok(())
}
