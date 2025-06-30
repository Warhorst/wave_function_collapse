use color_eyre::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use pad::{p, Position};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, List, Row, StatefulWidget, Table, TableState, Widget};
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

    let mut playground = Playground::default();

    let run_result = playground.run(terminal);
    ratatui::restore();
    Ok(run_result?)
}

#[derive(Default)]
struct State {
    stopped: bool,
    result_panel_state: ResultPanelState,
    settings_panel_state: SettingsPanelState
}

impl State {
    fn handle_key_input(&mut self, key_code: KeyCode) -> io::Result<()> {
        match key_code {
            KeyCode::Char('q') => {
                self.stopped = true
            },
            KeyCode::Char('c') => {
                self.result_panel_state.collapse(&self.settings_panel_state.settings);
            },
            KeyCode::Char('s') if !self.settings_panel_state.selected => {
                self.settings_panel_state.select();
                self.result_panel_state.deselect();
            },
            KeyCode::Char('r') if !self.result_panel_state.selected => {
                self.result_panel_state.select();
                self.settings_panel_state.deselect();
            },
            _ => {}
        }

        self.settings_panel_state.handle_key_inputs(key_code)?;

        Ok(())
    }
}

struct ResultPanelState {
    selected: bool,
    collapsed: Vec<(Position, Tile)>
}

impl ResultPanelState {
    fn select(&mut self) {
        self.selected = true;
    }

    fn deselect(&mut self) {
        self.selected = false;
    }

    fn collapse(&mut self, settings: &Settings) {
        self.collapsed = collapse(settings)
    }
}

impl Default for ResultPanelState {
    fn default() -> Self {
        ResultPanelState {
            selected: false,
            collapsed: vec![]
        }
    }
}

struct SettingsPanelState {
    selected: bool,
    settings: Settings,
    table_state: TableState
}

impl SettingsPanelState {
    fn handle_key_inputs(&mut self, key_code: KeyCode) -> io::Result<()> {
        if !self.selected {
            return Ok(())
        }

        match key_code {
            KeyCode::Up => {
                self.table_state.select_previous();
            },
            KeyCode::Down => {
                self.table_state.select_next()
            },
            KeyCode::Right => {
                if let Some(index) = self.table_state.selected() {
                    match index {
                        WIDTH_INDEX => self.settings.width += 1,
                        HEIGHT_INDEX => self.settings.height += 1,
                        _ => {}
                    }
                }
            },
            KeyCode::Left => {
                if let Some(index) = self.table_state.selected() {
                    match index {
                        WIDTH_INDEX => self.settings.width = self.settings.width.saturating_sub(1),
                        HEIGHT_INDEX => self.settings.height = self.settings.height.saturating_sub(1),
                        _ => {}
                    }
                }
            },
            _ => {}
        }

        Ok(())
    }

    fn select(&mut self) {
        self.selected = true;
        self.table_state.select(Some(0));
    }

    fn deselect(&mut self) {
        self.selected = false;
        self.table_state.select(None)
    }
}

impl Default for SettingsPanelState {
    fn default() -> Self {
        let mut table_state = TableState::new();
        table_state.select(Some(0));

        SettingsPanelState {
            selected: true,
            settings: Settings::default(),
            table_state
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

#[derive(Default)]
struct Playground {
    state: State,
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
        if let Event::Key(key_event) = event::read()? && key_event.kind == KeyEventKind::Press {
            self.state.handle_key_input(key_event.code)?;
        }

        Ok(())
    }
}

impl Widget for &mut Playground {
    fn render(self, area: Rect, buf: &mut Buffer) where Self: Sized {
        let vert_chunks = Layout::vertical([
            Constraint::Length(4),
            Constraint::Fill(1)
        ]).split(area);

        let hor_chunks = Layout::horizontal([
            Constraint::Percentage(75),
            Constraint::Percentage(25),
        ]).split(vert_chunks[1]);

        ControlsPanel.render(vert_chunks[0], buf, &mut self.state);
        ResultPanel.render(hor_chunks[0], buf, &mut self.state.result_panel_state);
        SettingsPanel.render(hor_chunks[1], buf, &mut self.state.settings_panel_state);
    }
}

/// Panel which shows the controls of the playground
struct ControlsPanel;

impl StatefulWidget for ControlsPanel {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let block = Block::bordered()
            .title(" Controls ")
            .border_set(border::DOUBLE);

        let items = vec![
            " <c> Collapse with current settings | <q> Quit ".to_string(),
            match state.result_panel_state.selected {
                true => "".to_string(),
                false => " <↓, ↑> Select setting | <←, →> Decrease / Increase value ".to_string()
            }
        ];

        Widget::render(
            List::new(items).block(block),
            area,
            buf
        )
    }
}

struct ResultPanel;

impl StatefulWidget for &ResultPanel {
    type State = ResultPanelState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) where Self: Sized {
        let border_set = if state.selected {
            border::THICK
        } else {
            border::PLAIN
        };
        let block = Block::bordered()
            .title(Line::from(" WFC Result <r> ").centered())
            .border_set(border_set);

        let inner = block.inner(area);

        block.render(area, buf);

        let mut sorted = state.collapsed.clone();
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
    type State = SettingsPanelState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) where Self: Sized {
        let border_set = if state.selected {
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

        StatefulWidget::render(table, inner, buf, &mut state.table_state);
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

fn collapse(
    settings: &Settings
) -> Vec<(Position, Tile)> {
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
        settings.width,
        settings.height,
        tiles,
    )
        .with_constraint(possible_neighbours)
        .with_seed(42)
        .collapse()
}
