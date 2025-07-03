use pad::{p, Position};
use ratatui::buffer::Buffer;
use ratatui::layout;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, Widget, StatefulWidget};
use ratatui::symbols::border;
use ratatui::widgets::Block;
use crate::{collapse, Settings, State, Tile};

pub struct ResultPanelState {
    pub selected: bool,
    pub collapsed: Vec<(Position, Tile)>
}

impl ResultPanelState {
    pub fn select(&mut self) {
        self.selected = true;
    }

    pub fn deselect(&mut self) {
        self.selected = false;
    }

    pub fn collapse(&mut self, settings: &Settings) {
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

pub struct ResultPanel;

impl StatefulWidget for &ResultPanel {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) where Self: Sized {
        let state = &state.result_panel_state;
        
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
