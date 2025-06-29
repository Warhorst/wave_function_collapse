use color_eyre::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use pad::{p, Position};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Row, StatefulWidget, Table, TableState, Widget};
use ratatui::{layout, DefaultTerminal};
use std::io;
use wave_function_collapse::{PossibleNeighbours, WaveFunctionCollapse};
use Tile::*;

/// Index of the settings row representing the width value
const WIDTH_INDEX: usize = 0;
/// Index of the settings row representing the height value
const HEIGHT_INDEX: usize = 1;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let mut playground = Playground {
        state: State::default(),
        result_panel: ResultPanel {
            collapsed: collapse()
        },
        settings_panel: SettingsPanel
    };

    let run_result = playground.run(terminal);
    ratatui::restore();
    Ok(run_result?)
}

struct State {
    stopped: bool,
    selected_panel: SelectedPanel,
    settings: Settings,
    settings_table: TableState,
}

impl Default for State {
    fn default() -> Self {
        let mut table_state = TableState::new();
        table_state.select(Some(0));

        State {
            stopped: false,
            selected_panel: SelectedPanel::Settings,
            settings: Settings::default(),
            settings_table: table_state
        }
    }
}

/// Settings for the wave function collapse which will be executed in the playground
struct Settings {
    width: usize,
    height: usize
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            width: 20,
            height: 20
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum SelectedPanel {
    Result,
    Settings
}

struct Playground {
    state: State,
    result_panel: ResultPanel,
    settings_panel: SettingsPanel
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
                match (key_event.code, self.state.selected_panel) {
                    (KeyCode::Char('q'), _) => {
                        self.state.stopped = true;
                    },
                    (KeyCode::Char('s'), SelectedPanel::Result) => {
                        self.state.settings_table.select(Some(0));
                        self.state.selected_panel = SelectedPanel::Settings
                    },
                    (KeyCode::Char('r'), SelectedPanel::Settings) => {
                        self.state.settings_table.select(None);
                        self.state.selected_panel = SelectedPanel::Result
                    },
                    (KeyCode::Up, SelectedPanel::Settings) => {
                        self.state.settings_table.select_previous();
                    },
                    (KeyCode::Down, SelectedPanel::Settings) => {
                        self.state.settings_table.select_next();
                    },
                    (KeyCode::Right, SelectedPanel::Settings) => {
                        if let Some(index) = self.state.settings_table.selected() {
                            match index {
                                WIDTH_INDEX => self.state.settings.width += 1,
                                HEIGHT_INDEX => self.state.settings.height += 1,
                                _ => {}
                            }
                        }
                    },
                    (KeyCode::Left, SelectedPanel::Settings) => {
                        if let Some(index) = self.state.settings_table.selected() {
                            match index {
                                WIDTH_INDEX => self.state.settings.width = self.state.settings.width.saturating_sub(1),
                                HEIGHT_INDEX => self.state.settings.height = self.state.settings.height.saturating_sub(1),
                                _ => {}
                            }
                        }
                    },
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

        self.result_panel.render(chunks[0], buf, &mut self.state);
        self.settings_panel.render(chunks[1], buf, &mut self.state);
    }
}

struct ResultPanel {
    collapsed: Vec<(Position, Tile)>
}

impl StatefulWidget for &ResultPanel {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) where Self: Sized {
        let border_set = if let SelectedPanel::Result = state.selected_panel {
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

struct SettingsPanel;

impl StatefulWidget for &SettingsPanel {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) where Self: Sized {
        let border_set = if let SelectedPanel::Settings = state.selected_panel {
            border::THICK
        } else {
            border::PLAIN
        };
        let block = Block::bordered()
            .title(" Settings <s> ")
            .border_set(border_set);
        let inner = block.inner(area);

        block.render(area, buf);

        let rows = [
            Row::new(vec!["Width".to_string(), format!("{}", state.settings.width)]),
            Row::new(vec!["Height".to_string(), format!("{}", state.settings.height)])
        ];
        let widths = [
            Constraint::Percentage(50),
            Constraint::Percentage(50)
        ];
        let table = Table::new(rows, widths)
            .row_highlight_style(Style::new().reversed())
            .highlight_symbol(">");

        StatefulWidget::render(table, inner, buf, &mut state.settings_table);
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
