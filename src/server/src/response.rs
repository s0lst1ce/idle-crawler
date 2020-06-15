use crate::buildings::{AllBuildings, Building, BuildingID, DependencyTree};
use crate::resources::ResourceID;
use crate::tile::Position;
use crate::tile::Tile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
enum Response {
    Event,
    Error,
}

#[derive(Debug, Deserialize, Serialize)]
enum Event {
    Build {
        pos: Position,
        building: BuildingID,
        amount: u32,
    },
    Demolish {
        pos: Position,
        building: BuildingID,
        amount: u32,
    },
    Hire {
        building: BuildingID,
        amount: u32,
    },
    Fire {
        building: BuildingID,
        amount: u32,
    },
    Deposit {
        resource: ResourceID,
        amount: u32,
    },
    Withdraw {
        resource: ResourceID,
        amount: u32,
    },
}

#[derive(Debug, Deserialize, Serialize)]
enum Error {
    InsufficientResource(ResourceID),
    InsufficientSlot(BuildingID),
    InsufficientStockpile(ResourceID),

    //auth errors
    InvalidToken,
    Unregistered,
    AlreadyRegistered,
}
