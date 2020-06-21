use crate::resources::ResourceID;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub type BuildingID = u8;

#[derive(Debug, Deserialize, Serialize)]
pub struct Building {
    pub name: String,
    //whether the building extracts natural resources
    pub extractor: bool,
    //Vector of the IDs of all buildings that the player must own to build this one
    pub prerequisites: Vec<BuildingID>,
    //resource_name, amount
    pub produced: HashMap<ResourceID, u32>,
    //resource_name, amount
    pub consumed: HashMap<ResourceID, u32>,
    pub max_workers: u32,
    //resource_name, amount
    pub construction_cost: HashMap<ResourceID, u32>,
}

pub type AllBuildings = HashMap<BuildingID, Building>;

//The first value is a map where the values are vecs of building (IDs) that depend on the resource key (ID)
//The second is collection of Building which have no deps.
#[derive(Debug)]
pub struct DependencyTree(
    pub HashMap<ResourceID, Vec<BuildingID>>,
    pub Vec<BuildingID>,
);
impl DependencyTree {
    fn new(buildings: &AllBuildings) -> DependencyTree {
        let mut tree: HashMap<ResourceID, Vec<BuildingID>> = HashMap::new();
        //This Vec holds all buildings without deps. This way they can be handled first and not stall the logic.
        //This also allowing gives a perf boost.
        let mut free = Vec::new();
        for (name, building) in buildings.iter() {
            if building.consumed.len() > 0 {
                for (resource, _) in building.consumed.iter() {
                    tree.entry(*resource).or_default().push(*name);
                }
            } else {
                free.push(*name);
            }
        }
        DependencyTree(tree, free)
    }
}

pub fn load_buildings<P: AsRef<Path>>(path: P) -> (AllBuildings, DependencyTree) {
    let file = std::fs::read(path).expect("couldn't read buildings.json");
    let data: AllBuildings =
        serde_json::from_slice(&file).expect("couldn't serialize buildings JSON");

    let tree = DependencyTree::new(&data);
    (data, tree)
}
