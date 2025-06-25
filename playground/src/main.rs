use color_eyre::Result;
use std::io;
use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
use ratatui::prelude::*;
use ratatui::symbols::border;
use ratatui::widgets::Block;

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let run_result = Playground::default().run(terminal);
    ratatui::restore();
    Ok(run_result?)
}

#[derive(Default)]
struct Playground {
    stopped: bool
}

impl Playground {
    fn run(&mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        while !self.stopped {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char('q') => {
                        self.stopped = true;
                    }
                    _ => {}
                }
            }
            _ => {}
        };
        Ok(())
    }
}

impl Widget for &Playground {
    fn render(self, area: Rect, buf: &mut Buffer) where Self: Sized {
        let title = Line::from("WFC Playground".bold());

        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::THICK);

        block.render(area, buf)
    }
}