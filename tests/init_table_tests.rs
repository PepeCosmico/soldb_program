mod utils;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program_test::*;
use solana_sdk::transport::TransportError;

use soldb_program::accounts::SolTable;

#[tokio::test]
async fn test_init_table_ok() -> Result<(), TransportError> {
    let (banks_client, payer, last_blockhash) = utils::setup().await?;

    let program_id = soldb_program::id();

    let name = "Test".to_string();
    let table = SolTable { name: name.clone() };
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
