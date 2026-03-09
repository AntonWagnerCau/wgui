use std::thread;
use std::time::{Duration, Instant};
use w_gui::AccentColor;

fn main() {
    env_logger::init();

    let mut ctx = w_gui::Context::new();

    // Simulation state
    let mut generation: u32;
    let mut fitness: f64;
    let mut iterations: u32;
    let mut training_time_secs = 273f64;
    let mut ai_enabled = true;
    let mut disturbance = false;
    
    // Performance metrics
    let mut fps: u32;
    let mut frame_time: f32;
    let mut population = 2378u32;
    
    // Progress values
    let mut training_progress: f64;
    let mut evolution_progress: f64;
    let mut resource_usage: f64;
    
    // Chart data (circular buffer for sparklines)
    let mut velocity_history: Vec<f32> = vec![0.0; 50];
    let mut fitness_history: Vec<f32> = vec![0.0; 50];
    let mut angle_history: Vec<f32> = vec![0.0; 50];
    
    // Plot data (longer history for larger plots)
    let mut fps_history: Vec<f32> = vec![60.0; 100];
    let mut memory_history: Vec<f32> = vec![256.0; 100];
    
    println!("Open the URL printed above in your browser.");
    println!("Dashboard demo showcasing grid layouts and plots.\n");

    let mut speed = 1.0f32;
    let mut last_update = Instant::now();

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last_update).as_secs_f32() * speed;
        last_update = now;

        // Update simulation values
        training_time_secs += dt as f64;
        
        // Simulate changing values
        generation = 154 + (training_time_secs as u32 / 5);
        fitness = 0.40 + (training_time_secs.sin() * 0.1 + 0.15).clamp(0.0, 0.95);
        iterations = 634 + (training_time_secs as u32 * 10);
        
        // Update FPS with some variation
        fps = (60.0 + (training_time_secs * 3.0).sin() * 10.0) as u32;
        frame_time = 1000.0 / fps as f32;
        
        // Update history for charts
        velocity_history.remove(0);
        velocity_history.push(10.0 + (training_time_secs as f32 * 2.0).sin() * 5.0);
        
        fitness_history.remove(0);
        fitness_history.push(fitness as f32 * 100.0);
        
        angle_history.remove(0);
        angle_history.push((training_time_secs as f32 * 1.5).sin() * 180.0);
        
        // Update plot data
        fps_history.remove(0);
        fps_history.push(fps as f32);
        memory_history.remove(0);
        memory_history.push(256.0 + (training_time_secs as f32 * 2.0).sin() * 50.0);
        
        // Update progress bars
        training_progress = ((training_time_secs % 10.0) / 10.0).clamp(0.0, 1.0);
        evolution_progress = (fitness).clamp(0.0, 1.0);
        resource_usage = (0.45 + (training_time_secs * 0.5).sin() * 0.3).clamp(0.0, 1.0);

        // ============================================
        // TOP STATS BAR - Key metrics in a grid
        // ============================================
        {
            let mut win = ctx.window("Training Status");
            win.set_accent(AccentColor::Coral);
            
            // Stats arranged in a 3-column grid
            win.grid(3, |grid| {
                grid.stat("Generation", &generation.to_string(), None, AccentColor::Coral);
                grid.stat("Fitness", &format!("{:.2}", fitness), Some("max 0.95"), AccentColor::Green);
                grid.stat("Iterations", &iterations.to_string(), None, AccentColor::Blue);
            });
            
            win.separator();
            
            // Status indicators in a 2-column grid
            win.grid(2, |grid| {
                grid.status(
                    "AI State",
                    ai_enabled,
                    Some("Enabled"),
                    Some("Disabled"),
                    AccentColor::Green,
                    AccentColor::Red,
                );
                grid.status(
                    "Disturbance",
                    disturbance,
                    Some("Active"),
                    Some("Inactive"),
                    AccentColor::Yellow,
                    AccentColor::Red,
                );
            });
        }

        // ============================================
        // PROGRESS PANEL - Training progress
        // ============================================
        {
            let mut win = ctx.window("Progress");
            win.set_accent(AccentColor::Teal);
            
            win.progress_bar_with_subtitle(
                "Training",
                training_progress,
                AccentColor::Teal,
                "Backpropagation phase",
            );
            
            win.progress_bar_with_subtitle(
                "Evolution",
                evolution_progress,
                AccentColor::Green,
                &format!("Generation {}", generation),
            );
            
            win.progress_bar_with_subtitle(
                "Resources",
                resource_usage,
                AccentColor::Blue,
                "GPU utilization",
            );
        }

        // ============================================
        // PERFORMANCE PANEL - Grid of stats + plots
        // ============================================
        {
            let mut win = ctx.window("Performance");
            win.set_accent(AccentColor::Purple);
            
            // Stats grid - 4 columns
            win.grid(4, |grid| {
                grid.stat("FPS", &fps.to_string(), Some(&format!("{:.1} ms", frame_time)), AccentColor::Green);
                grid.stat("Population", &population.to_string(), Some("entities"), AccentColor::Coral);
                grid.stat("Entities", "1024", Some("active"), AccentColor::Blue);
                grid.stat("Time", &format!("{:.0}s", training_time_secs), None, AccentColor::Yellow);
            });
            
            win.separator();
            
            // Larger plot showing performance over time
            win.plot(
                "Performance History",
                &[
                    ("FPS", &fps_history, AccentColor::Green),
                    ("Memory", &memory_history, AccentColor::Blue),
                ],
                Some("Time"),
                Some("Value"),
            );
            
            win.separator();
            
            // Mini charts in a grid
            win.grid(3, |grid| {
                grid.mini_chart("Velocity", &velocity_history, Some(" m/s"), AccentColor::Coral);
                grid.mini_chart("Fitness", &fitness_history, Some("%"), AccentColor::Green);
                grid.mini_chart("Angle", &angle_history, Some("°"), AccentColor::Blue);
            });
        }

        // ============================================
        // CONTROLS PANEL - Interactive widgets
        // ============================================
        {
            let mut win = ctx.window("Controls");
            win.set_accent(AccentColor::Blue);
            
            // Basic controls
            win.checkbox("Enable AI", &mut ai_enabled);
            win.checkbox("Apply Disturbance", &mut disturbance);
            
            win.separator();
            
            // Simulation speed slider
            win.slider("Sim Speed", &mut speed, 0.0..=5.0);
            
            // Population slider
            let mut pop_i32 = population as i32;
            win.slider_int("Population", &mut pop_i32, 100..=5000);
            population = pop_i32 as u32;
        }

        // ============================================
        // SOLUTIONS PANEL - Grid of progress bars
        // ============================================
        {
            let mut win = ctx.window("Top Solutions");
            win.set_accent(AccentColor::Green);
            
            // Solutions in a 2-column grid
            win.grid(2, |grid| {
                grid.progress_bar("Solution A", 0.26, AccentColor::Green);
                grid.progress_bar("Solution B", 0.25, AccentColor::Purple);
                grid.progress_bar("Solution C", 0.23, AccentColor::Blue);
                grid.progress_bar("Solution D", 0.21, AccentColor::Orange);
            });
        }

        ctx.end_frame();

        // Simulate 30 FPS
        thread::sleep(Duration::from_millis(33));
    }
}
