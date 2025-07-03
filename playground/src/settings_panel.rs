use crate::Settings;
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::prelude::{StatefulWidget, Style, Stylize, Widget};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Row, Table, TableState};
use std::io;
use crate::weights_dialog::{WeightsDialog, WeightsDialogState};

/// Index of the settings row representing the width value
const WIDTH_INDEX: usize = 0;
/// Index of the settings row representing the height value
const HEIGHT_INDEX: usize = 1;
const WEIGHTS_INDEX: usize = 2;

pub struct SettingsPanelState {
    pub selected: bool,
    pub table_state: TableState,
    pub weights_dialog_state: WeightsDialogState
}

impl SettingsPanelState {
    pub fn select(&mut self) {
        self.selected = true;
        self.table_state.select(Some(0));
    }

    pub fn deselect(&mut self) {
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
            table_state,
            weights_dialog_state: WeightsDialogState::default()
        }
    }
}

pub struct SettingsPanel;

impl SettingsPanel {
    pub fn handle_key_input(
        key_code: KeyCode,
        state: &mut SettingsPanelState,
        settings: &mut Settings
    ) -> io::Result<()> {
        if !state.selected {
            return Ok(())
        }

        if state.weights_dialog_state.open {
            return WeightsDialog::handle_key_input(
                key_code,
                &mut state.weights_dialog_state,
                settings
            );
        }

        match key_code {
            KeyCode::Up => {
                state.table_state.select_previous();
            },
            KeyCode::Down => {
                state.table_state.select_next()
            },
            KeyCode::Right => {
                if let Some(index) = state.table_state.selected() {
                    match index {
                        WIDTH_INDEX => settings.width += 1,
                        HEIGHT_INDEX => settings.height += 1,
                        _ => {}
                    }
                }
            },
            KeyCode::Left => {
                if let Some(index) = state.table_state.selected() {
                    match index {
                        WIDTH_INDEX => settings.width = settings.width.saturating_sub(1),
                        HEIGHT_INDEX => settings.height = settings.height.saturating_sub(1),
                        _ => {}
                    }
                }
            },
            KeyCode::Char(' ') => {
                if let Some(index) = state.table_state.selected() && index == WEIGHTS_INDEX {
                    state.weights_dialog_state.open = true
                }
            },
            _ => {}
        }

        Ok(())
    }
}

impl<'a> StatefulWidget for &'a SettingsPanel {
    type State = (&'a mut SettingsPanelState, &'a mut Settings);

    fn render(self, area: Rect, buf: &mut Buffer, (state, settings): &mut Self::State) where Self: Sized {
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
            Row::new(vec!["Width".to_string(), format!("{}", settings.width)]),
            Row::new(vec!["Height".to_string(), format!("{}", settings.height)]),
            Row::new(vec!["Weights...".to_string(), String::new()])
        ];
        let widths = [
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1)
        ];
        let table = Table::new(rows, widths)
            .row_highlight_style(Style::new().reversed())
            .highlight_symbol(">");

        StatefulWidget::render(table, inner, buf, &mut state.table_state);

        // todo maybe just dont use a dialog, everything is harder this way. Just make the settings
        //  accessible with indexes

        if state.weights_dialog_state.open {
            WeightsDialog.render(buf.area, buf, &mut (&mut state.weights_dialog_state, settings))
        }
    }
}
