mod weights_dialog;
mod result_panel;
mod settings_panel;
mod controls_panel;

use crate::controls_panel::*;
use crate::result_panel::*;
use crate::settings_panel::*;
use crate::weights_dialog::*;
use color_eyre::Result;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use pad::Position;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
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
    weights_dialog_state: WeightsDialogState
}

/// Settings for the wave function collapse which will be executed in the playground
pub struct Settings {
    width: usize,
    height: usize,
    weights: Vec<f32>
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            width: 20,
            height: 20,
            weights: vec![1.0; 3]
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
    ) -> io::Result<()> {
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
            KeyCode::Char('d') => {
                state.settings_panel_state.weight_dialog_open = !state.settings_panel_state.weight_dialog_open;
            },
            _ => {}
        }

        SettingsPanel::handle_key_input(
            key_code,
            &mut state.settings_panel_state,
            &mut state.settings
        )?;
        WeightsDialog::handle_key_input(
            key_code,
            &mut state.weights_dialog_state,
            &mut state.settings,
        )?;

        Ok(())
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
            Playground::handle_key_input(key_event.code, &mut self.state)?;
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
        SettingsPanel.render(hor_chunks[1], buf, &mut (&mut self.state.settings_panel_state, &mut self.state.settings));

        if self.state.settings_panel_state.weight_dialog_open {
            WeightsDialog.render(area, buf, &mut (&mut self.state.weights_dialog_state, &mut self.state.settings))
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
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
