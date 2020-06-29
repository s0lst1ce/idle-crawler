use crate::buildings::BuildingID;
use crate::player::Username;
use crate::resources::ResourceID;
use crate::tile::Position;
use crate::tile::Tile;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Response {
    Event(Event),
    Exception(Exception),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Event {
    Action(Action),
    World(World),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum World {
    GetTile(Position),
    Tile(Tile),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Action {
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
pub enum Exception {
    PlaceHolder,
    //related to player Action
    InsufficientResource(ResourceID),
    InsufficientSlot(BuildingID),
    InsufficientStockpile(ResourceID),
    NotFound,

    //world exploration
    TileNotOwned(Position),

    //auth errors
    InvalidToken,
    Unregistered,
    AlreadyRegistered,
}
