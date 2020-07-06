use crate::player::Username;
use crate::ResourceID;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceEntry {
    pub id: ResourceID,
    pub amount: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Offer {
    pub offering: Vec<ResourceEntry>,
    pub requesting: Vec<ResourceEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Trade {
    pub from: Username,
    pub to: Username,
    pub offer: Offer,
}
