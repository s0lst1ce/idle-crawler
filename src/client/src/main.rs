use core::Game;

fn main() {

    let mut game = Game::new(0);
    let username = "Player1";
    match game.add_player(String::from(username)) {
        Ok(_) => println!("Player {} added successfully", username),
        Err(_) => eprint!("Couldn't add player {}", username)
    };

    println!("Game object with 1 player:\n{:#?}", game);
}
