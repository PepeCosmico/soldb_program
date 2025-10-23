use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct SolTable {
    pub name: String,
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct SolValue {
    pub val: Vec<u8>,
}
