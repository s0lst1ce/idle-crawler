use server::{Game, AllBuildings};
use server::Player;

fn main() {

    let mut game = Game::new(0);
    game.add_player(String::from("Player1"));

    println!("Game object with 1 player:\n{:#?}", game);

}
