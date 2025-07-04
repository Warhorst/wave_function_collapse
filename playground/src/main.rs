mod weights_dialog;
mod result_panel;
mod settings_panel;
mod controls_panel;
mod seed_dialog;

use crate::controls_panel::*;
use crate::result_panel::*;
use crate::settings_panel::*;
use color_eyre::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use pad::Position;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::style::Color;
use ratatui::widgets::{StatefulWidget, Widget};
use ratatui::DefaultTerminal;
use std::io;
use wave_function_collapse::{PossibleNeighbours, WaveFunctionCollapse};
use Tile::*;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    let mut playground = Playground::default();

    let run_result = playground.run(terminal);
    ratatui::restore();
    Ok(run_result?)
}

#[derive(Default)]
pub struct State {
    stopped: bool,
    settings: Settings,
    result_panel_state: ResultPanelState,
    settings_panel_state: SettingsPanelState,
}

impl State {
    fn dialog_open(&self) -> bool {
        self.settings_panel_state.seed_dialog_state.open || self.settings_panel_state.weights_dialog_state.open
    }
}

/// Settings for the wave function collapse which will be executed in the playground
pub struct Settings {
    tiles: Vec<Tile>,
    width: usize,
    height: usize,
    weights: Vec<f32>,
    seed: String
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            tiles: vec![Water, Sand, Forest],
            width: 20,
            height: 20,
            weights: vec![1.0; 3],
            seed: "42".into()
        }
    }
}

#[derive(Default)]
struct Playground {
    state: State,
}

impl Playground {
    fn handle_key_input(
        key_code: KeyCode,
        state: &mut State
    ) {
        if !state.dialog_open() {
            match key_code {
                KeyCode::Char('q') => {
                    state.stopped = true
                },
                KeyCode::Char('c') => {
                    state.result_panel_state.collapse(&state.settings);
                },
                KeyCode::Char('s') if !state.settings_panel_state.selected => {
                    state.settings_panel_state.select();
                    state.result_panel_state.deselect();
                },
                KeyCode::Char('r') if !state.result_panel_state.selected => {
                    state.result_panel_state.select();
                    state.settings_panel_state.deselect();
                },
                _ => {}
            }
        }

        SettingsPanel::handle_key_input(key_code, state);
    }

    fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.state.stopped {
            terminal.draw(|frame| frame.render_widget(&mut *self, frame.area()))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key_event) = event::read()? && key_event.kind == KeyEventKind::Press {
            Playground::handle_key_input(key_event.code, &mut self.state);
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
        ResultPanel.render(hor_chunks[0], buf, &mut self.state);
        SettingsPanel.render(hor_chunks[1], buf, &mut self.state);
    }
}

/// Determine the area for some dialog
fn dialog_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tile {
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
    let tiles = settings.tiles.clone();
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
        .with_weights(settings.weights.iter().copied())
        .with_seed(settings.seed.clone())
        .collapse()
}
