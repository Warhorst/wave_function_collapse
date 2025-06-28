use color_eyre::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use pad::{p, Position};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Color;
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, List, StatefulWidget, Widget};
use ratatui::{layout, DefaultTerminal};
use std::io;
use wave_function_collapse::{PossibleNeighbours, WaveFunctionCollapse};
use Tile::*;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let mut playground = Playground {
        state: State {
            stopped: false,
            selected_screen: SelectedScreen::ControlScreen
        },
        result_screen: ResultScreen {
            collapsed: collapse()
        },
        control_screen: ControlScreen {
            setting_index: 0,
            width: 0,
            height: 0,
        }
    };

    let run_result = playground.run(terminal);
    ratatui::restore();
    Ok(run_result?)
}

struct State {
    stopped: bool,
    selected_screen: SelectedScreen
}

#[derive(Eq, PartialEq)]
enum SelectedScreen {
    ResultScreen,
    ControlScreen
}

struct Playground {
    state: State,
    result_screen: ResultScreen,
    control_screen: ControlScreen
}

impl Playground {
    fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.state.stopped {
            terminal.draw(|frame| frame.render_widget(&mut *self, frame.area()))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char('q') => {
                        self.state.stopped = true;
                    },
                    KeyCode::Char('c') => {
                        self.state.selected_screen = SelectedScreen::ControlScreen
                    },
                    KeyCode::Char('r') => {
                        self.state.selected_screen = SelectedScreen::ResultScreen
                    }
                    _ => {}
                }
            }
            _ => {}
        };
        Ok(())
    }
}

impl Widget for &mut Playground {
    fn render(self, area: Rect, buf: &mut Buffer) where Self: Sized {
        let chunks = Layout::horizontal([
            Constraint::Percentage(75),
            Constraint::Percentage(25),
        ]).split(area);

        self.result_screen.render(chunks[0], buf, &mut self.state);
        self.control_screen.render(chunks[1], buf, &mut self.state);
    }
}

struct ResultScreen {
    collapsed: Vec<(Position, Tile)>
}

impl StatefulWidget for &ResultScreen {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) where Self: Sized {
        let border_set = if let SelectedScreen::ResultScreen = state.selected_screen {
            border::THICK
        } else {
            border::PLAIN
        };
        let block = Block::bordered()
            .title(Line::from(" WFC Result <r> ").centered())
            .border_set(border_set);

        let inner = block.inner(area);

        block.render(area, buf);

        let mut sorted = self.collapsed.clone();
        sorted.sort_by(|(pos_a, _), (pos_b, _)| pos_a.cmp(pos_b));

        for (xi, x) in (inner.left()..inner.right()).enumerate() {
            for (yi, y) in (inner.top()..inner.bottom()).enumerate() {
                if let Some((_, tile)) = sorted.iter().find(|(pos, _)| *pos == p!(xi, yi)) {
                    let char = tile.get_char();
                    let color = tile.get_color();
                    buf[layout::Position::new(x, y)].set_char(char).set_fg(color);
                }
            }
        }
    }
}

struct ControlScreen {
    /// The index of the setting currently selected
    setting_index: usize,
    width: usize,
    height: usize
}

impl StatefulWidget for &ControlScreen {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) where Self: Sized {
        let border_set = if let SelectedScreen::ControlScreen = state.selected_screen {
            border::THICK
        } else {
            border::PLAIN
        };
        let block = Block::bordered()
            .title(" Control Panel <c> ")
            .border_set(border_set);
        let inner = block.inner(area);

        block.render(area, buf);

        let list = List::new([
            "Foo",
            "Bar",
            "Baz"
        ]);

        Widget::render(list, inner, buf);
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum Tile {
    Water,
    Sand,
    Forest
}

impl Tile {
    fn get_char(&self) -> char {
        match self {
            Water => 'W',
            Sand => 'S',
            Forest => 'F'
        }
    }

    fn get_color(&self) -> Color {
        match self {
            Water => Color::Blue,
            Sand => Color::Yellow,
            Forest => Color::Green
        }
    }
}

fn collapse() -> Vec<(Position, Tile)> {
    let tiles = vec![Water, Sand, Forest];
    let possible_neighbours = PossibleNeighbours::new([
          (Water, Water),
          (Water, Sand),
          (Sand, Water),
          (Sand, Sand),
          (Sand, Forest),
          (Forest, Sand),
          (Forest, Forest),
    ], &tiles);

    WaveFunctionCollapse::<3, Tile>::new(
        20,
        20,
        tiles,
    )
        .with_constraint(possible_neighbours)
        .with_seed(42)
        .collapse()
}
