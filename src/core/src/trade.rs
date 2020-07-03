use crate::player::Username;
use crate::ResourceID;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct ResourceEntry {
    id: ResourceID,
    amount: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Offer {
    offering: Vec<ResourceEntry>,
    requesting: Vec<ResourceEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Trade {
    from: Username,
    offer: Offer,
}
