use serde::{Serialize, Deserialize};
use bincode;

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    pub vk: Vec<u8>,
    pub proof_data: Vec<u8>,
    pub common: Vec<u8>
}

impl Transaction {
    pub fn serialize(&self) -> Vec<u8> {
        bincode::serialize(self).expect("Failed to serialize transaction")
    }

    pub fn deserialize(data: &[u8]) -> Self {
        bincode::deserialize(data).expect("Failed to deserialize transaction")
    }
}
