#[allow(unused_imports)]
mod ui;
use core::{AllBuildings, Game, Player};
use crossterm::event::{self, Event as TEvent, KeyCode};
use std::{
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
    thread::spawn({
        let mut last_tick = Instant::now();
        move || loop {
            if event::poll(tick_rate - last_tick.elapsed()).unwrap() {
                //currently only handling key events
                if let TEvent::Key(key) = event::read().unwrap() {
                    tx.send(key);
                }
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
    }
    Ok(())
}
