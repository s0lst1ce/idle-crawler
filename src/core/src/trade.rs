use crate::player::Username;
use crate::ResourceID;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Hash, PartialEq)]
pub struct ResourceEntry {
    pub id: ResourceID,
    pub amount: u32,
}

/// Trade offer
///
/// This struct is used as a mean to convey a deal between players.
/// It allows them to formally exhange resources. The party making the offer will
/// present its offer in "offering" and its requests in "requesting".
///
/// Note that either of these can be empty. This allows for donations.
/// However special care should be used so that no empty offers are made.
#[derive(Debug, Serialize, Deserialize, PartialEq, Hash)]
pub struct Offer {
    pub offering: Vec<ResourceEntry>,
    pub requesting: Vec<ResourceEntry>,
}

/// Trade register
///
/// The word "ledger" refers to an account books. As such it is used to keep track of an entity's
/// open trades. This can be either an alliance, or more commonly, a plasyer.
#[derive(Debug, Serialize, Deserialize)]
pub struct Ledger {
    pub inbound: HashMap<Username, Vec<Offer>>,
    pub outbound: HashMap<Username, Vec<Offer>>,
}

impl Ledger {
    pub fn new() -> Ledger {
        Ledger {
            inbound: HashMap::new(),
            outbound: HashMap::new(),
        }
    }
}
