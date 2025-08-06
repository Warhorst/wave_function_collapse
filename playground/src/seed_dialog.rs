use appcui::prelude::*;

#[ModalWindow(events=[ButtonEvents],response=String)]
pub struct SeedDialog {
    seed_input: Handle<TextField>,
    confirm_button: Handle<Button>
}

impl SeedDialog {
    pub fn new(seed: String) -> Self {
        let mut window = SeedDialog {
            base: ModalWindow::new("Set Seed", layout!("align:c,w:20%,h:20%"), window::Flags::None),
            seed_input: Handle::None,
            confirm_button: Handle::None
        };
        
        let seed_input = TextField::new(&seed, layout!("x:0,y:0%,w:100%,h:50%"), textfield::Flags::None);
        window.seed_input = window.add(seed_input);
        
        let confirm_button = button!("Confirm,x:0,y:50%,w:100%,h:50%");
        window.confirm_button = window.add(confirm_button);
        
        window
    }
}

impl ButtonEvents for SeedDialog {
    fn on_pressed(&mut self, handle: Handle<Button>) -> EventProcessStatus {
        if handle == self.confirm_button {
            let seed_input = self.control(self.seed_input).unwrap();
            let seed = seed_input.text().to_string();
            self.exit_with(seed)
        }
        
        EventProcessStatus::Processed
    }
}