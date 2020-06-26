use crate::buildings::BuildingID;
use crate::player::Username;
use crate::resources::ResourceID;
use crate::tile::Position;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Response {
    Event(Event),
    Exception(Exception),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Event {
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
    Discover,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Exception {
    //related to player actions
    InsufficientResource(ResourceID),
    InsufficientSlot(BuildingID),
    InsufficientStockpile(ResourceID),
    NotFound,
    PlaceHolder,

    //auth errors
    InvalidToken,
    Unregistered,
    AlreadyRegistered,
}
