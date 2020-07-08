use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResourceID(pub u8);

pub type ResourceName = String;
pub type AllResources = HashMap<ResourceID, ResourceName>;

pub fn load_resources<P: AsRef<Path>>(path: P) -> AllResources {
    let file = std::fs::read(path).expect("couldn't read resources.json");
    let data: AllResources =
        serde_json::from_slice(&file).expect("couldn't serialize resources JSON");
    data
}
