mod seed_dialog;

use Tile::*;

use appcui::prelude::*;
use pad::Position;
use wave_function_collapse::{PossibleNeighbours, WaveFunctionCollapse};
use crate::seed_dialog::SeedDialog;

fn main() -> Result<(), Error> {
    let mut app = App::new()
        .single_window()
        .theme(Theme::new(Themes::Light))
        .build()?;
    let win = PlaygroundWindow::new();

    app.add_window(win);
    app.run();
    Ok(())
}

#[Window(events=[ButtonEvents,NumericSelectorEvents<usize>])]
struct PlaygroundWindow {
    settings: Settings,
    results_canvas: Handle<Canvas>,
    width_selector: Handle<NumericSelector<usize>>,
    height_selector: Handle<NumericSelector<usize>>,
    seed_label: Handle<Label>,
    seed_button: Handle<Button>,
    create_button: Handle<Button>
}

impl NumericSelectorEvents<usize> for PlaygroundWindow {
    fn on_value_changed(
        &mut self,
        handle: Handle<NumericSelector<usize>>,
        value: usize
    ) -> EventProcessStatus {
        if handle == self.width_selector {
            self.settings.width = value
        } else if handle == self.height_selector {
            self.settings.height = value
        }
        
        EventProcessStatus::Processed
    }
}

impl PlaygroundWindow {
    fn new() -> Self {
        let mut window = PlaygroundWindow {
            base: Window::new(
                "Playground",
                Layout::new("d:c,w:100,h:100"),
                window::Flags::NoCloseButton
            ),
            settings: Settings::default(),
            results_canvas: Handle::None,
            width_selector: Handle::None,
            height_selector: Handle::None,
            seed_label: Handle::None,
            seed_button: Handle::None,
            create_button: Handle::None
        };

        let mut results_panel = panel!("Results,x:0%,y:0%,w:80%,h:100%,type:Border");

        let canvas = canvas!("50x50,x:0,y:0,w:100%,h:100%");
        window.results_canvas = results_panel.add(canvas);
        
        let mut settings_panel = panel!("Settings,x:80%,y:0%,w:20%,h:95%,type:Border");
        
        settings_panel.add(label!("Width,x:0,y:0,w:50%"));
        window.width_selector = settings_panel.add(NumericSelector::<usize>::new(
            window.settings.width,
            5,
            50,
            1,
            Layout::new("x:50%,y:0,w:50%"),
            numericselector::Flags::None
        ));
        settings_panel.add(label!("Height,x:0,y:1,w:50%"));
        window.height_selector = settings_panel.add(NumericSelector::<usize>::new(
            window.settings.height,
            5,
            50,
            1,
            Layout::new("x:50%,y:1,w:50%"),
            numericselector::Flags::None
        ));

        let seed_label = Label::new(&window.settings.seed, Layout::new("x:0,y:2,w:50%"));
        window.seed_label = settings_panel.add(seed_label);
        let mut seed_button = button!("caption='Set Seed...',x:50%,y:2,w:50%");
        seed_button.set_hotkey(key!("S"));
        window.seed_button = settings_panel.add(seed_button);
        
        let mut create_button = button!("Create,x:80%,y:95%,w:20%");
        create_button.set_hotkey(key!("C"));
        window.create_button = window.add(create_button);
        
        window.add(results_panel);
        window.add(settings_panel);
        
        window
    }
}

impl ButtonEvents for PlaygroundWindow {
    fn on_pressed(&mut self, handle: Handle<Button>) -> EventProcessStatus {
        if handle == self.create_button {
            let collapsed = collapse(&self.settings);
            let canvas_handle = self.results_canvas;
            let canvas = self.control_mut(canvas_handle).unwrap();
            let surface = canvas.drawing_surface_mut();
            surface.clear(Character::new(' ', Color::Transparent, Color::Transparent, CharFlags::None));

            for (pos, tile) in collapsed {
                surface.write_char(
                    pos.x as i32,
                    pos.y as i32,
                    Character::new(
                        tile.get_char(),
                        tile.get_color(),
                        Color::Transparent,
                        CharFlags::None
                    )
                )
            }
        } else if handle == self.seed_button {
            if let Some(seed) = SeedDialog::new(self.settings.seed.clone()).show() {
                self.settings.seed = seed.clone();

                let label_handle = self.seed_label;
                let label = self.control_mut(label_handle).unwrap();
                label.set_caption(&seed)
            }
        }

        EventProcessStatus::Processed
    }
}

pub struct Settings {
    tiles: Vec<Tile>,
    width: usize,
    height: usize,
    weights: Vec<f32>,
    seed: String
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            tiles: vec![Water, Sand, Forest],
            width: 20,
            height: 20,
            weights: vec![1.0; 3],
            seed: "42".into()
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Tile {
    Water,
    Sand,
    Forest
}

impl Tile {
    fn get_char(&self) -> char {
        match self {
            Water => 'W',
            Sand => 'S',
            Forest => 'F'
        }
    }

    fn get_color(&self) -> Color {
        match self {
            Water => Color::Blue,
            Sand => Color::Yellow,
            Forest => Color::Green
        }
    }
}

fn collapse(
    settings: &Settings
) -> Vec<(Position, Tile)> {
    let tiles = settings.tiles.clone();
    let possible_neighbours = PossibleNeighbours::new([
          (Water, Water),
          (Water, Sand),
          (Sand, Water),
          (Sand, Sand),
          (Sand, Forest),
          (Forest, Sand),
          (Forest, Forest),
      ], &tiles);

    WaveFunctionCollapse::<3, Tile>::new(
        settings.width,
        settings.height,
        tiles,
    )
        .with_constraint(possible_neighbours)
        .with_weights(settings.weights.iter().copied())
        .with_seed(settings.seed.clone())
        .collapse()
}
