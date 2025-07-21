use appcui::prelude::*;
use crate::Tile;

// todo cannot use the generic type directly in the response. Create issue.
pub type WeightsDialogResponse = Vec<f32>;

#[ModalWindow(events=[ButtonEvents], response=WeightsDialogResponse)]
pub struct WeightsDialog {
    weight_setters: Vec<Handle<NumericSelector<f32>>>,
    confirm_button: Handle<Button>
}

impl WeightsDialog {
    pub fn new(weights: impl IntoIterator<Item=(Tile, f32)>) -> Self {
        let weights = weights.into_iter().collect::<Vec<_>>();
        let mut window = WeightsDialog {
            base: ModalWindow::new(
                "Set Weights",
                Layout::new(&format!("d:c,w:20%,h:{}", weights.len() + 3)), // +3 because 2 for the border and 1 extra for the button
                window::Flags::None
            ),
            weight_setters: Vec::with_capacity(weights.len()),
            confirm_button: Handle::None
        };

        for (i, (tile, weight)) in weights.iter().enumerate() {
            window.add(Label::new(
                &format!("{tile:?}"),
                Layout::new(&format!("x:0,y:{i},w:50%"))
            ));
            let selector = NumericSelector::new(
                *weight,
                0.25,
                5.0,
                0.25,
                Layout::new(&format!("x:50%,y:{i},w:50%")),
                numericselector::Flags::None
            );
            let handle = window.add(selector);
            window.weight_setters.push(handle);
        }

        window.confirm_button = window.add(Button::new("Confirm", Layout::new(&format!("x:0,y:{},w:100%", weights.len())), button::Type::Normal));

        window
    }
}

impl ButtonEvents for WeightsDialog {
    fn on_pressed(&mut self, handle: Handle<Button>) -> EventProcessStatus {
        if handle == self.confirm_button {
            let response = self.weight_setters
                .iter()
                .map(|handle| self.control(*handle).unwrap())
                .map(|selector| selector.value())
                .collect::<Vec<_>>();
            self.exit_with(response);
        }

        EventProcessStatus::Processed
    }
}