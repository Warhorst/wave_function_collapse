use crossterm::event::{Event, KeyCode, KeyEvent, read};
use pad::position::Position;
use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Flex, Layout},
    prelude::{Buffer, Rect},
    style::{Color, Style},
    symbols::border,
    text::Line,
    widgets::{Block, Cell, Clear, Row, StatefulWidget, Table, TableState, Widget},
};
use ratatui_tools::tilemap::{Char, TileMap};
use wave_function_collapse::{WfcBuilder, cell::DynCell, constraints::PossibleNeighbours};

// TODO Some ideas to further improve the playground and even make it a standalone application:
// - Add a seed input (requires a text input widget)
// - Allow step-by-step collapsing
// - Add a cursor to the map to collapse specific positions
// - Add scrollbars to the map to allow bigger boards
// - Load tiles from files
// - Load constraints from files

const HIGHLIGHT_STYLE: Style = Style::new().fg(Color::Green);

pub fn main() -> std::io::Result<()> {
    ratatui::run(|t| App::default().run(t))
}

enum Message {
    /// Input was processed, but no message was created
    None,
    /// Closes the playground
    CloseApp,
    /// Increment the width of the board
    IncrementWidth,
    /// Decrements the width of the board
    DecrementWidth,
    /// Increments the height of the board
    IncrementHeight,
    /// Decrements the height of the board
    DecrementHeight,
    /// Opens the [WeightDialog]
    OpenWeightDialog,
    /// Closes the [WeightDialog]
    CloseWeightDialog,
    /// Increment the weight of the tile at the given index
    IncrementWeight(usize),
    /// Decrement the weight of the tile at the given index
    DecrementWeight(usize),
    /// Perform a wave function collapse with the current settings
    Collapse,
}

struct State {
    /// Tells if the app should close
    should_exit: bool,
    /// All the tiles in the Wfc system
    tiles: Vec<Tile>,
    /// The weights of the tiles in the wfc system
    weights: Vec<f32>,
    /// The width of the Wfc board
    width: usize,
    /// The height of the Wfc board
    height: usize,
    /// The seed which is used in the Wfc
    seed: String,
    /// All currently collapsed positions
    collapsed_positions: Vec<(Position, Tile)>,
}

impl Default for State {
    fn default() -> Self {
        State {
            should_exit: false,
            tiles: vec![Tile::Water, Tile::Sand, Tile::Forest],
            weights: vec![1.0, 1.0, 1.0],
            width: State::MIN_WIDTH_HEIGHT,
            height: State::MIN_WIDTH_HEIGHT,
            seed: "42".into(),
            collapsed_positions: vec![],
        }
    }
}

impl State {
    /// Minimum width and height of the board
    const MIN_WIDTH_HEIGHT: usize = 20;
    /// Maximum widtht and height of the board
    const MAX_WIDTH_HEIGHT: usize = 50;
    /// The minimum weight of a tile
    const MIN_WEIGHT: f32 = 0.25;
    /// The maximum weight of a tile
    const MAX_WEIGHT: f32 = 3.0;
    /// The step used to increment / decrement a tile
    const WEIGHT_STEP: f32 = 0.25;

    fn increment_width(&mut self) {
        Self::increment_value(&mut self.width);
    }

    fn increment_height(&mut self) {
        Self::increment_value(&mut self.height);
    }

    fn increment_value(value: &mut usize) {
        if *value < Self::MAX_WIDTH_HEIGHT {
            *value += 5
        }
    }

    fn decrement_widht(&mut self) {
        Self::decrement_value(&mut self.width);
    }

    fn decrement_height(&mut self) {
        Self::decrement_value(&mut self.height);
    }

    fn decrement_value(value: &mut usize) {
        if *value > Self::MIN_WIDTH_HEIGHT {
            *value -= 5
        }
    }

    fn increment_weight(
        &mut self,
        index: usize,
    ) {
        if let Some(weight) = self.weights.get_mut(index)
            && *weight < Self::MAX_WEIGHT
        {
            *weight += Self::WEIGHT_STEP
        }
    }

    fn decrement_weight(
        &mut self,
        index: usize,
    ) {
        if let Some(weight) = self.weights.get_mut(index)
            && *weight > Self::MIN_WEIGHT
        {
            *weight -= Self::WEIGHT_STEP
        }
    }
}

struct App {
    state: State,
    wfc_map: WfcMap,
    settings_panel: SettingsPanel,
    weight_dialog: Option<WeightDialog>,
}

impl Default for App {
    fn default() -> Self {
        App {
            state: State::default(),
            wfc_map: WfcMap { selected: false },
            settings_panel: SettingsPanel::default(),
            weight_dialog: None,
        }
    }
}

impl App {
    fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> std::io::Result<()> {
        while !self.state.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut *self, frame.area()))?;

            if let Event::Key(key) = read()?
                && key.is_press()
            {
                match self.handle_events(key) {
                    Message::None => {}
                    Message::CloseApp => self.state.should_exit = true,
                    Message::IncrementWidth => self.state.increment_width(),
                    Message::DecrementWidth => self.state.decrement_widht(),
                    Message::IncrementHeight => self.state.increment_height(),
                    Message::DecrementHeight => self.state.decrement_height(),
                    Message::OpenWeightDialog => self.weight_dialog = Some(WeightDialog::default()),
                    Message::CloseWeightDialog => self.weight_dialog = None,
                    Message::IncrementWeight(index) => self.state.increment_weight(index),
                    Message::DecrementWeight(index) => self.state.decrement_weight(index),
                    Message::Collapse => self.state.collapsed_positions = self.collapse(),
                }
            }
        }

        Ok(())
    }

    fn handle_events(
        &mut self,
        event: KeyEvent,
    ) -> Message {
        if let Some(dialog) = &mut self.weight_dialog {
            return dialog.handle_events(event, &self.state);
        }

        match event.code {
            KeyCode::Char('1') => {
                self.wfc_map.selected = true;
                self.settings_panel.selected = false;
                Message::None
            }
            KeyCode::Char('2') => {
                self.settings_panel.selected = true;
                self.wfc_map.selected = false;
                Message::None
            }
            KeyCode::Esc => Message::CloseApp,
            KeyCode::Char('c') => Message::Collapse,
            _ => self.settings_panel.handle_events(event),
        }
    }

    fn collapse(&self) -> Vec<(Position, Tile)> {
        use Tile::*;

        let tiles = self.state.tiles.clone();
        let possible_neighbours = PossibleNeighbours::new(
            [
                (Water, Water),
                (Water, Sand),
                (Sand, Water),
                (Sand, Sand),
                (Sand, Forest),
                (Forest, Sand),
                (Forest, Forest),
            ],
            &tiles,
        );

        WfcBuilder::<Tile, DynCell>::new(self.state.width, self.state.height, tiles)
            .with_constraint(possible_neighbours)
            .with_weights(self.state.weights.iter().copied())
            .with_seed(42)
            .build()
            .unwrap()
            .collapse()
            .unwrap()
    }
}

impl Widget for &mut App {
    fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
    ) where
        Self: Sized,
    {
        let block = Block::bordered()
            .title_top(Line::from(" Playground ").centered())
            .title_top(Line::from(" Close <Esc> ").right_aligned())
            .border_set(border::THICK);

        let layout = Layout::horizontal([Constraint::Percentage(80), Constraint::Percentage(20)]);
        let [map_area, control_area] = layout.areas(block.inner(area));

        block.render(area, buf);

        self.wfc_map.render(map_area, buf, &mut self.state);
        self.settings_panel
            .render(control_area, buf, &mut self.state);

        if let Some(dialog) = &mut self.weight_dialog {
            dialog.render(area, buf, &mut self.state);
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tile {
    Water,
    Sand,
    Forest,
}

struct WfcMap {
    selected: bool,
}

impl StatefulWidget for &WfcMap {
    type State = State;

    fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
    ) where
        Self: Sized,
    {
        let style = match self.selected {
            true => HIGHLIGHT_STYLE,
            false => Style::default(),
        };

        let tile_map = TileMap::default().paint(|tiles| {
            for (pos, tile) in &state.collapsed_positions {
                let color = match tile {
                    Tile::Water => Color::Blue,
                    Tile::Sand => Color::Yellow,
                    Tile::Forest => Color::Green,
                };

                tiles.set_tile(pos.x, pos.y, Char::new(' ', Color::White).bg(color));
            }
        });

        let block = Block::bordered()
            .title_top(" Wfc Map <1> ")
            .title_bottom(" Collapse <c> ")
            .border_style(style);

        let inner = block.inner(area);

        block.render(area, buf);
        tile_map.render(inner, buf);
    }
}

// TODO to fully use the elm architecture, make the settings a stateful widget and introduce the settings state
//  Or maybe don't, as the settings would not have any fields left? Maybe thats alright, as a widget should only
//  have other widgets as fields

struct SettingsPanel {
    selected: bool,
    table_state: TableState,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        SettingsPanel {
            selected: false,
            table_state: TableState::default().with_selected(0),
        }
    }
}

impl SettingsPanel {
    const NUM_SETTINGS: usize = 4;
    const WIDTH_INDEX: usize = 0;
    const HEIGHT_INDEX: usize = 1;
    const WEIGHT_INDEX: usize = 3;

    fn handle_events(
        &mut self,
        event: KeyEvent,
    ) -> Message {
        if !self.selected {
            return Message::None;
        }

        match event.code {
            KeyCode::Char('k') => {
                self.previous_setting();
                Message::None
            }
            KeyCode::Char('j') => {
                self.next_setting();
                Message::None
            }
            KeyCode::Char('h') => self.decrement(),
            KeyCode::Char('l') => self.increment(),
            KeyCode::Char(' ') => self.open_dialog(),
            _ => Message::None,
        }
    }

    fn previous_setting(&mut self) {
        let index = match self.table_state.selected() {
            Some(index) => {
                if index > 0 {
                    index - 1
                } else {
                    return;
                }
            }
            None => 0,
        };
        self.table_state.select(Some(index));
    }

    fn next_setting(&mut self) {
        let index = match self.table_state.selected() {
            Some(index) => {
                if index < Self::NUM_SETTINGS - 1 {
                    index + 1
                } else {
                    return;
                }
            }
            None => 0,
        };
        self.table_state.select(Some(index));
    }

    fn increment(&self) -> Message {
        if let Some(index) = self.table_state.selected() {
            if index == Self::WIDTH_INDEX {
                return Message::IncrementWidth;
            } else if index == Self::HEIGHT_INDEX {
                return Message::IncrementHeight;
            }
        }

        Message::None
    }

    fn decrement(&self) -> Message {
        if let Some(index) = self.table_state.selected() {
            if index == Self::WIDTH_INDEX {
                return Message::DecrementWidth;
            } else if index == Self::HEIGHT_INDEX {
                return Message::DecrementHeight;
            }
        }

        Message::None
    }

    fn open_dialog(&self) -> Message {
        if let Some(index) = self.table_state.selected()
            && index == Self::WEIGHT_INDEX
        {
            return Message::OpenWeightDialog;
        }

        Message::None
    }
}

impl StatefulWidget for &mut SettingsPanel {
    type State = State;

    fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
    ) where
        Self: Sized,
    {
        let style = match self.selected {
            true => HIGHLIGHT_STYLE,
            false => Style::default(),
        };

        let block = Block::bordered()
            .title_top(" Controls <2> ")
            .border_style(style);

        let rows = [
            Row::new([Cell::new("Width"), Cell::new(state.width.to_string())]),
            Row::new([Cell::new("Height"), Cell::new(state.height.to_string())]),
            Row::new([Cell::new("Seed"), Cell::new(state.seed.clone())]),
            Row::new([Cell::new("Weights"), Cell::default()]),
        ];

        let widths = [Constraint::Percentage(50), Constraint::Percentage(50)];

        let table = Table::new(rows, widths).row_highlight_style(style);

        let block_area = block.inner(area);
        block.render(area, buf);

        StatefulWidget::render(table, block_area, buf, &mut self.table_state);
    }
}

struct WeightDialog {
    table_state: TableState,
}

impl Default for WeightDialog {
    fn default() -> Self {
        WeightDialog {
            table_state: TableState::default().with_selected(Some(0)),
        }
    }
}

impl WeightDialog {
    const HEIGHT_PERCENTAGE: u16 = 20;
    const WIDTH_PERCENTAGE: u16 = 30;

    fn handle_events(
        &mut self,
        event: KeyEvent,
        state: &State,
    ) -> Message {
        match event.code {
            KeyCode::Esc => Message::CloseWeightDialog,
            KeyCode::Char('j') => {
                self.next_weight(state);
                Message::None
            }
            KeyCode::Char('k') => {
                self.previous_weight();
                Message::None
            }
            KeyCode::Char('l') => self.increment_weight(),
            KeyCode::Char('h') => self.decrement_weight(),
            _ => Message::None,
        }
    }

    fn next_weight(
        &mut self,
        state: &State,
    ) {
        if let Some(index) = self.table_state.selected()
            && index < state.tiles.len()
        {
            self.table_state.select(Some(index + 1))
        }
    }

    fn previous_weight(&mut self) {
        if let Some(index) = self.table_state.selected()
            && index > 0
        {
            self.table_state.select(Some(index - 1))
        }
    }

    fn increment_weight(&self) -> Message {
        if let Some(index) = self.table_state.selected() {
            Message::IncrementWeight(index)
        } else {
            Message::None
        }
    }

    fn decrement_weight(&self) -> Message {
        if let Some(index) = self.table_state.selected() {
            Message::DecrementWeight(index)
        } else {
            Message::None
        }
    }

    fn dialog_area(parent_area: Rect) -> Rect {
        let vertical =
            Layout::vertical([Constraint::Percentage(Self::HEIGHT_PERCENTAGE)]).flex(Flex::Center);
        let horizontal =
            Layout::horizontal([Constraint::Percentage(Self::WIDTH_PERCENTAGE)]).flex(Flex::Center);
        let [area] = vertical.areas(parent_area);
        let [area] = horizontal.areas(area);
        area
    }
}

impl StatefulWidget for &mut WeightDialog {
    type State = State;

    fn render(
        self,
        area: Rect,
        buf: &mut Buffer,
        state: &mut Self::State,
    ) where
        Self: Sized,
    {
        let area = WeightDialog::dialog_area(area);

        Clear.render(area, buf);

        let block = Block::bordered().title_top(Line::from("Weights").centered());
        let inner = block.inner(area);

        block.render(area, buf);

        let rows = state.tiles.iter().enumerate().map(|(i, t)| {
            Row::new([
                Cell::new(format!("{t:?}")),
                Cell::new(format!("{}", state.weights[i])),
            ])
        });
        let widhts = [Constraint::Percentage(50), Constraint::Percentage(50)];

        let table = Table::new(rows, widhts).row_highlight_style(HIGHLIGHT_STYLE);
        StatefulWidget::render(table, inner, buf, &mut self.table_state);
    }
}
