use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct SolTable {
    pub name: String,
    pub keys: Vec<String>,
}

impl SolTable {
    pub fn new(name: String) -> Self {
        Self {
            name,
            keys: Vec::new(),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct SolValue {
    pub val: Vec<u8>,
}
