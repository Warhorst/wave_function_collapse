use std::io;
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};
use ratatui::prelude::{Widget, StatefulWidget, Style, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Row, Table, TableState};
use crate::Settings;

/// Index of the settings row representing the width value
const WIDTH_INDEX: usize = 0;
/// Index of the settings row representing the height value
const HEIGHT_INDEX: usize = 1;

pub struct SettingsPanelState {
    pub selected: bool,
    pub table_state: TableState,
    pub weight_dialog_open: bool
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
            weight_dialog_open: false
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
            Row::new(vec!["Height".to_string(), format!("{}", settings.height)])
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
