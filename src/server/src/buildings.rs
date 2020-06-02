use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
pub struct Building {
    pub produced: HashMap<String, u32>,
    pub consumed: HashMap<String, u32>,
    pub max_workers: u32,
    pub requisites: HashMap<String, u32>,
}

pub fn load_buildings() -> HashMap<String, Building> {
    //currently hardcoded, change in the future
    let mut path = PathBuf::new();
    path.push("../../utilities/buildings.json");
    let mut file = String::new();
    File::open(path)
        .expect("Couldn't read file")
        .read_to_string(&mut file)
        .expect("Couldn't read as String.");
    let data: HashMap<String, Building> = serde_json::from_str(&file).expect("Valid JSON");
    data
}
