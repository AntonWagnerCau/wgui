//! wgui — a lightweight immediate-mode debug GUI served over localhost.
//! Visual theme heavily inspired by https://www.youtube.com/@PezzzasWork
//! # Example
//! ```no_run
//! let mut ctx = wgui::Context::new();
//! let mut color = [1.0f32, 0.0, 0.5];
//! let mut speed = 5.0f32;
//!
//! loop {
//!     let mut win = ctx.window("Utils");
//!     win.color_picker("My Color", &mut color);
//!     win.slider("Speed", &mut speed, 0.0..=10.0);
//!     drop(win);
//!     ctx.end_frame();
//!     // ... your game/engine frame ...
//! }
//! ```

mod context;
mod element;
mod protocol;
mod server;
mod window;

pub use context::{Context, ContextOptions};
pub use element::AccentColor;
pub use window::{Grid, Window};
