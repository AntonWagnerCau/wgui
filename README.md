# w-gui

A lightweight immediate-mode debug GUI served over localhost.

**[Documentation](https://antonwagnercau.github.io/wgui/)** — guide, widget reference and API docs.

## Overview

w-gui provides an in-process GUI framework for real-time parameter updates and visualization. Access the interface through a web browser on localhost.
The visual design is heavily inspired by https://www.youtube.com/@PezzzasWork

## Features

- Immediate-mode API
- WebSocket-based communication
- Color pickers
- Sliders
- Windows and panels
- Tabs (pages of windows)
- Zero external UI dependencies

## Usage

```rust
let mut ctx = w_gui::Context::new();
let mut color = [1.0f32, 0.0, 0.5];
let mut speed = 5.0f32;

loop {
    let mut win = ctx.window("Utils");
    win.color_picker("My Color", &mut color);
    win.slider("Speed", &mut speed, 0.0..=10.0);
    drop(win);
    ctx.end_frame();
    // ... your application frame ...
}
```

Access the GUI at `http://localhost:9080` (port configurable).

### Tabs

Windows can be grouped into tabs (pages); the browser shows a tab bar and only
the active tab's windows. Windows created directly on the context show on
every tab.

```rust
let mut tab = ctx.tab("Simulation");
let mut win = tab.window("Fire");
win.slider("Heat", &mut heat, 0.0..=10.0);
drop(win);
let mut win = tab.window("Wind");
win.slider("Strength", &mut wind, 0.0..=5.0);
```

## Requirements

- Rust 1.56+

## Examples

See `examples/` for dashboard and demo implementations.
