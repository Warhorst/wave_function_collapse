use crate::{dialog_area, State};
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::{Line, StatefulWidget, Widget};
use ratatui::text::Text;
use ratatui::widgets::{Block, Clear};

#[derive(Default)]
pub struct SeedDialogState {
    pub open: bool
}

/// Dialog for entering the wfc seed
pub struct SeedDialog;

impl SeedDialog {
    pub fn handle_key_input(
        key_code: KeyCode,
        state: &mut State,
    ) {
        let settings = &mut state.settings;
        let state = &mut state.settings_panel_state.seed_dialog_state;
        
        match key_code {
            KeyCode::Esc => {
                state.open = false
            },
            KeyCode::Backspace => {
                settings.seed.clear(); 
            },
            KeyCode::Char(char) => {
                settings.seed.push(char)
            },
            _ => {}
        }
    }
}

impl StatefulWidget for &SeedDialog {
    type State = State;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let settings = &state.settings;
        
        let area = dialog_area(area, 20, 20);

        let block = Block::bordered()
            .title("Seed")
            .title(Line::from("<Esc> close").right_aligned())
            .title_bottom("<Backspace> delete");
        let inner = block.inner(area);
        Clear.render(area, buf);
        block.render(area, buf);
        
        let text = Text::from(settings.seed.clone());
        text.render(inner, buf);
    }
}