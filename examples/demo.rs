use std::thread;
use std::time::Duration;
use w_gui::AccentColor;

fn main() {
    // Init logging so you can see w_gui server messages
    env_logger::init();

    let mut ctx = w_gui::Context::new();

    // -- state --
    let mut color = [0.2f32, 0.5, 0.8];
    let mut color4 = [1.0f32, 0.3, 0.6, 1.0];
    let mut speed = 5.0f32;
    let mut count = 10i32;
    let mut enabled = true;
    let mut name = String::from("Player1");
    let mut mode: usize = 0;
    let modes = ["Easy", "Normal", "Hard", "Nightmare"];
    let mut click_count = 0u32;

    println!("Open the URL printed above in your browser.");
    println!("Press Ctrl+C to quit.\n");

    loop {
        // -- "Rendering" window --
        {
            let mut win = ctx.window("Rendering");
            win.label("Render settings");
            win.separator();
            if win.color_picker("Sky Color", &mut color).changed() {
                println!("Sky Color: [{:.3}, {:.3}, {:.3}]", color[0], color[1], color[2]);
            }
            if win.color_picker4("Fog Color", &mut color4).changed() {
                println!("Fog Color: [{:.3}, {:.3}, {:.3}, {:.3}]", color4[0], color4[1], color4[2], color4[3]);
            }
            if win.slider("Speed", &mut speed, 0.0..=20.0).changed() {
                println!("Speed: {speed:.2}");
            }
            if win.slider_int("Ray Count", &mut count, 1..=256).changed() {
                println!("Ray Count: {count}");
            }
            if win.checkbox("Enable GI", &mut enabled).changed() {
                println!("Enable GI: {enabled}");
            }
        }

        // -- "Game" window --
        {
            let mut win = ctx.window("Game");
            if win.text_input("Player Name", &mut name).changed() {
                println!("Player Name: {name}");
            }
            if win.dropdown("Difficulty", &mut mode, &modes).changed() {
                println!("Difficulty: {}", modes[mode]);
            }
            win.separator();
            if win.button("Button").clicked() {
                speed = 5.0;
                count = 10;
                enabled = true;
                color = [0.2, 0.5, 0.8];
                click_count += 1;
            }
            win.label(&format!("Button pressed {} time(s)", click_count));
        }

        // -- "Stats" window - demonstrates grid layout --
        {
            let mut win = ctx.window("Stats");
            win.set_accent(AccentColor::Coral);
            
            // Stats arranged in a grid instead of stacked
            win.grid(2, |grid| {
                grid.stat("FPS", "60", Some("avg 58"), AccentColor::Green);
                grid.stat("Frame Time", "16.7", Some("ms"), AccentColor::Blue);
                grid.stat("Draw Calls", "1,024", None, AccentColor::Coral);
                grid.stat("Triangles", "45K", None, AccentColor::Purple);
            });
            
            win.separator();
            
            // Status indicators in a grid
            win.grid(2, |grid| {
                grid.status("Online", true, Some("Yes"), Some("No"), AccentColor::Green, AccentColor::Red);
                grid.status("Recording", false, Some("On"), Some("Off"), AccentColor::Yellow, AccentColor::Red);
            });
        }

        ctx.end_frame();

        // Simulate a ~30fps game loop
        thread::sleep(Duration::from_millis(33));
    }
}
