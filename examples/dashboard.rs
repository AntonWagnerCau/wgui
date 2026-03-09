use std::thread;
use std::time::{Duration, Instant};
use wgui::AccentColor;

fn main() {
    env_logger::init();

    let mut ctx = wgui::Context::new();

    // Simulation state (similar to the AI training examples)
    let start_time = Instant::now();
    let mut generation = 154u32;
    let mut fitness = 0.40f64;
    let mut iterations = 634u32;
    let mut training_time_secs = 273f64;
    let mut ai_enabled = true;
    let mut disturbance = false;
    
    // Performance metrics
    let mut fps = 60u32;
    let mut frame_time = 2.6f32;
    let mut population = 2378u32;
    
    // Progress values
    let mut training_progress = 0.27f64;
    let mut evolution_progress = 0.63f64;
    let mut resource_usage = 0.45f64;
    
    // Chart data (circular buffer for sparklines)
    let mut velocity_history: Vec<f32> = vec![0.0; 50];
    let mut fitness_history: Vec<f32> = vec![0.0; 50];
    let mut angle_history: Vec<f32> = vec![0.0; 50];
    
    println!("Open the URL printed above in your browser.");
    println!("Dashboard demo showcasing the new themed UI components.\n");

    let mut last_update = Instant::now();

    loop {
        let now = Instant::now();
        let dt = now.duration_since(last_update).as_secs_f32();
        last_update = now;

        // Update simulation values
        training_time_secs += dt as f64;
        
        // Simulate changing values
        generation = 154 + (training_time_secs as u32 / 5);
        fitness = 0.40 + (training_time_secs.sin() * 0.1 + 0.15).clamp(0.0, 0.95);
        iterations = 634 + (training_time_secs as u32 * 10);
        
        // Update history for charts
        velocity_history.remove(0);
        velocity_history.push(10.0 + (training_time_secs as f32 * 2.0).sin() * 5.0);
        
        fitness_history.remove(0);
        fitness_history.push(fitness as f32 * 100.0);
        
        angle_history.remove(0);
        angle_history.push((training_time_secs as f32 * 1.5).sin() * 180.0);
        
        // Update progress bars
        training_progress = ((training_time_secs % 10.0) / 10.0).clamp(0.0, 1.0);
        evolution_progress = (fitness).clamp(0.0, 1.0);
        resource_usage = (0.45 + (training_time_secs * 0.5).sin() * 0.3).clamp(0.0, 1.0);

        // ============================================
        // TOP STATS BAR - Key metrics
        // ============================================
        {
            let mut win = ctx.window("Training Status");
            win.set_accent(AccentColor::Coral);
            
            // Section header for stats
            win.section("Generation Stats");
            
            // Main metrics as stat cards
            win.stat("Generation", &generation.to_string(), None, AccentColor::Coral);
            win.stat("Fitness", &format!("{:.2}", fitness), Some("max 0.95"), AccentColor::Green);
            win.stat("Iterations", &iterations.to_string(), None, AccentColor::Blue);
            
            win.separator();
            
            // Status indicators
            win.status(
                "AI State",
                ai_enabled,
                Some("Enabled"),
                Some("Disabled"),
                AccentColor::Green,
                AccentColor::Red,
            );
            win.status(
                "Disturbance",
                disturbance,
                Some("Active"),
                Some("Inactive"),
                AccentColor::Yellow,
                AccentColor::Red,
            );
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
        // PERFORMANCE PANEL - Stats and charts
        // ============================================
        {
            let mut win = ctx.window("Performance");
            win.set_accent(AccentColor::Purple);
            
            // Stats grid
            win.section("Current Metrics");
            win.stat("FPS", &fps.to_string(), Some(&format!("{:.1} ms", frame_time)), AccentColor::Green);
            win.stat("Population", &population.to_string(), Some("entities"), AccentColor::Coral);
            win.stat("Time", &format!("{:.1}s", training_time_secs), None, AccentColor::Yellow);
            
            win.separator();
            
            // Mini charts
            win.section("Live Charts");
            win.mini_chart("Velocity", &velocity_history, Some(" m/s"), AccentColor::Coral);
            win.mini_chart("Fitness", &fitness_history, Some("%"), AccentColor::Green);
            win.mini_chart("Angle", &angle_history, Some("°"), AccentColor::Blue);
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
            let mut speed = 1.0f32;
            win.slider("Sim Speed", &mut speed, 0.0..=5.0);
            
            // Population slider
            let mut pop_slider = population as f32;
            let mut pop_i32 = population as i32;
            win.slider_int("Population", &mut pop_i32, 100..=5000);
            population = pop_i32 as u32;
        }

        // ============================================
        // SOLUTIONS PANEL - Example of solution cards
        // ============================================
        {
            let mut win = ctx.window("Top Solutions");
            win.set_accent(AccentColor::Green);
            
            // Simulated solution progress bars (like image 2)
            win.progress_bar_with_subtitle(
                "Solution 4",
                0.26,
                AccentColor::Green,
                "Avg speed: 11 cm/s",
            );
            win.progress_bar_with_subtitle(
                "Solution 3",
                0.25,
                AccentColor::Purple,
                "Avg speed: 12 cm/s",
            );
            win.progress_bar_with_subtitle(
                "Solution 1",
                0.23,
                AccentColor::Blue,
                "Avg speed: 10 cm/s",
            );
            win.progress_bar_with_subtitle(
                "Solution 2",
                0.23,
                AccentColor::Orange,
                "Avg speed: 10 cm/s",
            );
        }

        ctx.end_frame();

        // Simulate 30 FPS
        thread::sleep(Duration::from_millis(33));
    }
}
