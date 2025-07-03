use std::io;
use crossterm::event::KeyCode;
use crate::Settings;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::{StatefulWidget, Style, Stylize, Widget};
use ratatui::widgets::{Block, Clear, Row, Table, TableState};

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
        state: &mut WeightsDialogState,
        settings: &mut Settings
    ) -> io::Result<()> {
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
    type State = (&'a mut WeightsDialogState, &'a mut Settings);

    fn render(self, area: Rect, buf: &mut Buffer, (state, settings): &mut Self::State) {
        let area = self.dialog_area(area, 20, 40);

        let block = Block::bordered().title("Weights");
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