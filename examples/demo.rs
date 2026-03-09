use std::thread;
use std::time::Duration;

fn main() {
    // Init logging so you can see wgui server messages
    env_logger::init();

    let mut ctx = wgui::Context::new();

    // -- state --
    let mut color = [0.2f32, 0.5, 0.8];
    let mut prev_color = color;
    let mut color4 = [1.0f32, 0.3, 0.6, 1.0];
    let mut prev_color4 = color4;
    let mut speed = 5.0f32;
    let mut prev_speed = speed;
    let mut count = 10i32;
    let mut prev_count = count;
    let mut enabled = true;
    let mut prev_enabled = enabled;
    let mut name = String::from("Player1");
    let mut prev_name = name.clone();
    let mut mode: usize = 0;
    let mut prev_mode = mode;
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
            win.color_picker("Sky Color", &mut color);
            win.color_picker4("Fog Color", &mut color4);
            win.slider("Speed", &mut speed, 0.0..=20.0);
            win.slider_int("Ray Count", &mut count, 1..=256);
            win.checkbox("Enable GI", &mut enabled);
        }

        // -- "Game" window --
        {
            let mut win = ctx.window("Game");
            win.text_input("Player Name", &mut name);
            win.dropdown("Difficulty", &mut mode, &modes);
            win.separator();
            if win.button("Reset Settings") {
                speed = 5.0;
                count = 10;
                enabled = true;
                color = [0.2, 0.5, 0.8];
                click_count += 1;
            }
            win.label(&format!("Reset pressed {} time(s)", click_count));
        }

        ctx.end_frame();

        // Log state changes
        if color != prev_color {
            println!("Sky Color: [{:.3}, {:.3}, {:.3}]", color[0], color[1], color[2]);
            prev_color = color;
        }
        if color4 != prev_color4 {
            println!("Fog Color: [{:.3}, {:.3}, {:.3}, {:.3}]", color4[0], color4[1], color4[2], color4[3]);
            prev_color4 = color4;
        }
        if speed != prev_speed {
            println!("Speed: {speed:.2}");
            prev_speed = speed;
        }
        if count != prev_count {
            println!("Ray Count: {count}");
            prev_count = count;
        }
        if enabled != prev_enabled {
            println!("Enable GI: {enabled}");
            prev_enabled = enabled;
        }
        if name != prev_name {
            println!("Player Name: {name}");
            prev_name = name.clone();
        }
        if mode != prev_mode {
            println!("Difficulty: {}", modes[mode]);
            prev_mode = mode;
        }

        // Simulate a ~30fps game loop
        thread::sleep(Duration::from_millis(33));
    }
}
