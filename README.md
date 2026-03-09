# wgui

A lightweight immediate-mode debug GUI served over localhost.

## Overview

wgui provides an in-process GUI framework for real-time parameter updates and visualization. Access the interface through a web browser on localhost.
The visual design is heavily inspired by https://www.youtube.com/@PezzzasWork

## Features

- Immediate-mode API
- WebSocket-based communication
- Color pickers
- Sliders
- Windows and panels
- Zero external UI dependencies

## Usage

```rust
let mut ctx = wgui::Context::new();
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

## Requirements

- Rust 1.56+

## Examples

See `examples/` for dashboard and demo implementations.
