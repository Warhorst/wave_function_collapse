use crate::State;
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::{Line, StatefulWidget, Style, Stylize, Widget};
use ratatui::widgets::{Block, Clear, Row, Table, TableState};
use std::io;

pub struct WeightsDialogState {
    pub open: bool,
    table_state: TableState
}

impl Default for WeightsDialogState {
    fn default() -> Self {
        let mut ts = TableState::new();
        ts.select(Some(0));

        WeightsDialogState {
            open: false,
            table_state: ts
        }
    }
}

pub struct WeightsDialog;

impl WeightsDialog {
    pub fn handle_key_input(
        key_code: KeyCode,
        state: &mut State
    ) -> io::Result<()> {
        let settings = &mut state.settings;
        let state = &mut state.settings_panel_state.weights_dialog_state;
        
        match key_code {
            KeyCode::Up => {
                state.table_state.select_previous()
            },
            KeyCode::Down => {
                state.table_state.select_next()
            },
            KeyCode::Left => {
                if let Some(i) = state.table_state.selected() && let Some(value) = settings.weights.get_mut(i) {
                    *value -= 0.25;
                }
            },
            KeyCode::Right => {
                if let Some(i) = state.table_state.selected() && let Some(value) = settings.weights.get_mut(i) {
                    *value += 0.25;
                }
            },
            KeyCode::Esc => {
                state.open = false
            },
            _ => {}
        }

        Ok(())
    }

    fn dialog_area(&self, area: Rect, percent_x: u16, percent_y: u16) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }
}

impl<'a> StatefulWidget for &'a WeightsDialog {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let settings = &state.settings;
        let state = &mut state.settings_panel_state.weights_dialog_state;
        
        let area = self.dialog_area(area, 20, 40);

        let block = Block::bordered()
            .title("Weights")
            .title(Line::from("<Esc> close").right_aligned());
        let inner = block.inner(area);
        Clear.render(area, buf);
        block.render(area, buf);

        let widths = (0..settings.tiles.len())
            .into_iter()
            .map(|_| Constraint::Fill(1));
        let rows = settings.tiles
            .iter()
            .zip(settings.weights.iter())
            .map(|(tile, weight)| Row::new(vec![format!("{tile:?}"), format!("{weight}")]));

        let table = Table::new(rows, widths)
            .row_highlight_style(Style::new().reversed())
            .highlight_symbol(">");
        StatefulWidget::render(table, inner, buf, &mut state.table_state)
    }
}