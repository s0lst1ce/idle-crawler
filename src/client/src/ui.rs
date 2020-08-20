use crossterm::event::KeyCode;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
    Frame,
};

pub fn draw_main_menu<B: Backend>(f: &mut Frame<B>, area: Rect) {
    let mut menu = Menu::new(vec![
        (Choice::new("load", Message::PlaceHolder), None),
        (Choice::new("lew", Message::PlaceHolder), None),
        (Choice::new("join", Message::PlaceHolder), None),
        (Choice::new("host", Message::PlaceHolder), None),
        (Choice::new("quit", Message::PlaceHolder), None),
    ]);
    f.render_stateful_widget(menu.list, area, &mut menu.state);
}

#[derive(Debug)]
pub enum Message {
    PlaceHolder,
    Input(KeyCode),
    NextIteration,
}

#[derive(Debug)]
struct Choice {
    name: String,
    message: Message,
}

impl Choice {
    fn new(name: &str, message: Message) -> Choice {
        Choice {
            name: name.to_string(),
            message,
        }
    }

    //we change the displayed string to account for the shortcut
    fn set_short(&mut self, letter: char) {
        assert!(self.name.to_ascii_lowercase().contains(letter));
        self.name = self.name.replacen(
            letter,
            format!("({})", letter.to_ascii_uppercase()).as_ref(),
            1,
        );
    }
}

#[derive(Debug)]
struct Menu<'a> {
    buttons: Vec<Choice>,
    shortcuts: Vec<char>,
    list: List<'a>,
    state: ListState,
}

impl Menu<'_> {
    fn new(items: Vec<(Choice, Option<char>)>) -> Menu<'static> {
        let (buttons, shortcuts) = Menu::extrapolate_shorts(items);
        let list = Menu::new_list(&buttons);
        Menu {
            buttons,
            shortcuts,
            list,
            state: ListState::default(),
        }
    }

    fn new_list(buttons: &[Choice]) -> List<'static> {
        List::new(
            buttons
                .iter()
                .map(|b| ListItem::new(Span::raw(b.name.clone())))
                .collect::<Vec<ListItem>>(),
        )
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol("=>")
    }

    fn extrapolate_shorts(items: Vec<(Choice, Option<char>)>) -> (Vec<Choice>, Vec<char>) {
        let mut used_chars = Vec::with_capacity(items.len());
        let mut choices = Vec::with_capacity(items.len());
        let mut chars = Vec::with_capacity(items.len());

        for (mut choice, short) in items {
            chars.push(match short {
                //a shortcut is explicitely specified
                Some(letter) => {
                    if used_chars.contains(&letter) {
                        panic!("Two options can't have the same shortcut!");
                    } else {
                        used_chars.push(letter);
                        letter
                    }
                }

                //we need to assign a shortcut to the choice
                None => {
                    let mut found = char::default();
                    for l in choice.name.chars() {
                        let letter = l.to_ascii_lowercase();
                        if used_chars.contains(&letter) {
                            continue;
                        } else {
                            found = letter;
                            break;
                        }
                    }
                    if found == char::default() {
                        panic!("All letters are being used! No random selection available. Failed to assign a shortcut to the choice!");
                    }
                    found
                }
            });
            choice.set_short(*chars.last().unwrap());
            choices.push(choice);
        }
        (choices, chars)
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.buttons.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.buttons.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}
