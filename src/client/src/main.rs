use core::{AllBuildings, Game, Player};
use std::sync::mpsc::{channel, Receiver, Sender};

fn main() {
    let (tx1, rx1) = channel();
    let (tx2, rx2) = channel();
    let mut game = Game::new(0, (tx2, rx1));
    game.add_player(String::from("Player1"));

    println!("Game object with 1 player:\n{:#?}", game);
}
