use crate::resources::ResourceID;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

///A u8 that represents a unique Building.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct BuildingID(pub u8);

///Core component of a player's empire. Generates its resources.
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

///Unique mapping of all BuildindID and their corresponding Building.
pub type AllBuildings = HashMap<BuildingID, Building>;

///This struct enables Generator from the player module to create GenMap.
///
///It maps all buildings to the resources they consume to produce their goods. This means that a building will be maped to as many keys (ResourceID) as resources it consumes.
///This is the most optimal way that was found to generate the GenMaps and then the resources. This is not meant to be generated manually but read from the game data by serde_json.
///It was called DependencyTree because it serves a somewhat similar purpose. However it does differ in design a bit as a branch does not lead to another. Rather all branch "sprout" are keys. This is however enough and enables faster and simpler operations.
//The first value is a map where the values are vecs of building (IDs) that depend on the resource key (ID)
//The second is collection of Building which have no deps.
#[derive(Debug)]
pub struct DependencyTree(
    //The tree-like mapping. Where all sprouts are placed along-side one another.
    pub HashMap<ResourceID, Vec<BuildingID>>,
    //Some buildings do not consume any resource. This means that they wouldn't be taken into account by the Generator. To prevent this and reduce the slight offset caused by the tree in these buildings this "free" field is used and is always handled first.
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
