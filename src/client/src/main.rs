#[allow(unused_imports)]
mod ui;
use core::{AllBuildings, Game, Player};
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent};
use std::{
    error::Error,
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Clear},
    Terminal,
};
use ui::Message;

//the idea is to split the update logic in three parts:
// - event processing (aka: event loop)
// - game update
// - gui update

fn main() -> Result<(), io::Error> {
    //GUI setup
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    //handling events
    let (tx, rx) = mpsc::channel();

    //tick rate used for event lookup, not for the game
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            if event::poll(tick_rate - last_tick.elapsed()).unwrap() {
                //currently only handling key events
                if let CEvent::Key(key_event) = event::read().unwrap() {
                    tx.send(Message::Input(key_event.code));
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Message::NextIteration);
                last_tick = Instant::now();
            }
        }
    });

    loop {
        terminal.draw(|mut f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(80),
                        Constraint::Percentage(10),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            f.render_widget(Clear, f.size());
            let block = Block::default().title("Block").borders(Borders::ALL);
            f.render_widget(block, chunks[0]);
            ui::draw_main_menu(f, chunks[1]);
        })?;

        if let Ok(_e) = rx.recv() {
            println!("Dealing");
            match rx.recv().unwrap() {
                //we need to free resources to update the screen
                Message::NextIteration => (),
                Message::Input(key) => match key {
                    Up => println!("HEy!"),
                    Down => todo!(),
                    _ => println!("Nope"),
                },
                _ => unreachable!(),
            }
        }
    }
    Ok(())
}
