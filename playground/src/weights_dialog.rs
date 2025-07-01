use std::io;
use crossterm::event::KeyCode;
use crate::Settings;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Flex, Layout, Rect};
use ratatui::prelude::{StatefulWidget, Widget};
use ratatui::widgets::{Block, Clear, TableState};

pub struct WeightsDialogState {
    table_state: TableState
}

impl Default for WeightsDialogState {
    fn default() -> Self {
        WeightsDialogState {
            table_state: TableState::new()
        }
    }
}

// todo will be the dialog the user can set the weights for the different tiles
pub struct WeightsDialog;

impl WeightsDialog {
    pub fn handle_key_input(
        key_code: KeyCode,
        state: &mut WeightsDialogState,
        settings: &mut Settings
    ) -> io::Result<()> {
        // todo
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

    fn render(self, area: Rect, buf: &mut Buffer, (state, settigns): &mut Self::State) {
        let area = self.dialog_area(area, 60, 20);

        let block = Block::bordered().title("The Dialog");
        Clear.render(area, buf);
        block.render(area, buf);
    }
}

