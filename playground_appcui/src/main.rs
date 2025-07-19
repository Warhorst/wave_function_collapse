use Tile::*;

use appcui::prelude::*;

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

#[Window]
struct PlaygroundWindow {
    settings: Settings,
    width_selector: Handle<NumericSelector<usize>>,
    height_selector: Handle<NumericSelector<usize>>,
    create_button: Handle<Button>
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
            width_selector: Handle::None,
            height_selector: Handle::None,
            create_button: Handle::None
        };

        let results_panel = panel!("Results,x:0%,y:0%,w:80%,h:100%,type:Border");
        
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
        
        window.create_button = window.add(button!("Create,x:80%,y:95%,w:20%"));   
        
        window.add(results_panel);
        window.add(settings_panel);
        
        window
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
