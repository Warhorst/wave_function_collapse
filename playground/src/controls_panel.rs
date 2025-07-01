use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{StatefulWidget, Widget};
use ratatui::symbols::border;
use ratatui::widgets::{Block, List};
use crate::State;

/// Panel which shows the controls of the playground
pub struct ControlsPanel;

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
