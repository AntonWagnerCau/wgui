use std::ops::RangeInclusive;
use std::sync::Arc;

use crate::context::Context;
use crate::element::{AccentColor, ElementDecl, ElementKind, ElementMeta, PlotSeries, Value};

/// Response from a widget interaction.
pub struct Response {
    clicked: bool,
    changed: bool,
}

impl Response {
    /// Returns `true` whenever the widget was interacted with since the last update.
    pub fn clicked(&self) -> bool {
        self.clicked
    }

    /// Returns `true` only if the widget's value actually changed since the last update.
    pub fn changed(&self) -> bool {
        self.changed
    }
}

// ── Element declaration builders ─────────────────────────────────────
// Shared by Window and Grid to avoid duplicating ElementDecl construction.

fn build_slider(
    id: String,
    window: Arc<str>,
    label: &str,
    value: f32,
    range: &RangeInclusive<f32>,
) -> ElementDecl {
    ElementDecl {
        id,
        kind: ElementKind::Slider,
        label: label.to_string(),
        value: Value::Float(value as f64),
        meta: ElementMeta {
            min: Some(*range.start() as f64),
            max: Some(*range.end() as f64),
            step: Some(0.01),
            ..Default::default()
        },
        window,
    }
}

fn build_slider_int(
    id: String,
    window: Arc<str>,
    label: &str,
    value: i32,
    range: &RangeInclusive<i32>,
) -> ElementDecl {
    ElementDecl {
        id,
        kind: ElementKind::Slider,
        label: label.to_string(),
        value: Value::Int(value as i64),
        meta: ElementMeta {
            min: Some(*range.start() as f64),
            max: Some(*range.end() as f64),
            step: Some(1.0),
            ..Default::default()
        },
        window,
    }
}

fn build_checkbox(id: String, window: Arc<str>, label: &str, value: bool) -> ElementDecl {
    ElementDecl {
        id,
        kind: ElementKind::Checkbox,
        label: label.to_string(),
        value: Value::Bool(value),
        meta: ElementMeta::default(),
        window,
    }
}

fn build_color3(id: String, window: Arc<str>, label: &str, value: [f32; 3]) -> ElementDecl {
    ElementDecl {
        id,
        kind: ElementKind::ColorPicker3,
        label: label.to_string(),
        value: Value::Color3(value),
        meta: ElementMeta::default(),
        window,
    }
}

fn build_color4(id: String, window: Arc<str>, label: &str, value: [f32; 4]) -> ElementDecl {
    ElementDecl {
        id,
        kind: ElementKind::ColorPicker4,
        label: label.to_string(),
        value: Value::Color4(value),
        meta: ElementMeta::default(),
        window,
    }
}

fn build_text_input(id: String, window: Arc<str>, label: &str, value: &str) -> ElementDecl {
    ElementDecl {
        id,
        kind: ElementKind::TextInput,
        label: label.to_string(),
        value: Value::String(value.to_string()),
        meta: ElementMeta::default(),
        window,
    }
}

fn build_dropdown(
    id: String,
    window: Arc<str>,
    label: &str,
    selected: usize,
    options: &[&str],
) -> ElementDecl {
    ElementDecl {
        id,
        kind: ElementKind::Dropdown,
        label: label.to_string(),
        value: Value::Enum {
            selected,
            options: options.iter().map(|s| s.to_string()).collect(),
        },
        meta: ElementMeta::default(),
        window,
    }
}

fn build_button(id: String, window: Arc<str>, label: &str) -> ElementDecl {
    ElementDecl {
        id,
        kind: ElementKind::Button,
        label: label.to_string(),
        value: Value::Button(false),
        meta: ElementMeta::default(),
        window,
    }
}

fn build_label(id: String, window: Arc<str>, text: &str) -> ElementDecl {
    ElementDecl {
        id,
        kind: ElementKind::Label,
        label: text.to_string(),
        value: Value::String(text.to_string()),
        meta: ElementMeta::default(),
        window,
    }
}

fn build_progress_bar(
    id: String,
    window: Arc<str>,
    label: &str,
    value: f64,
    accent: AccentColor,
    subtitle: Option<&str>,
) -> ElementDecl {
    ElementDecl {
        id,
        kind: ElementKind::ProgressBar,
        label: label.to_string(),
        value: Value::Progress(value.clamp(0.0, 1.0)),
        meta: ElementMeta {
            accent: Some(accent.as_str().to_string()),
            subtitle: subtitle.map(|s| s.to_string()),
            ..Default::default()
        },
        window,
    }
}

fn build_stat(
    id: String,
    window: Arc<str>,
    label: &str,
    value: &str,
    subvalue: Option<&str>,
    accent: AccentColor,
) -> ElementDecl {
    ElementDecl {
        id,
        kind: ElementKind::Stat,
        label: label.to_string(),
        value: Value::StatValue {
            value: value.to_string(),
            subvalue: subvalue.map(|s| s.to_string()),
        },
        meta: ElementMeta {
            accent: Some(accent.as_str().to_string()),
            ..Default::default()
        },
        window,
    }
}

fn build_status(
    id: String,
    window: Arc<str>,
    label: &str,
    active: bool,
    active_text: Option<&str>,
    inactive_text: Option<&str>,
    active_color: AccentColor,
    inactive_color: AccentColor,
) -> ElementDecl {
    ElementDecl {
        id,
        kind: ElementKind::Status,
        label: label.to_string(),
        value: Value::StatusValue {
            active,
            active_text: active_text.map(|s| s.to_string()),
            inactive_text: inactive_text.map(|s| s.to_string()),
            active_color: Some(active_color.as_str().to_string()),
            inactive_color: Some(inactive_color.as_str().to_string()),
        },
        meta: ElementMeta::default(),
        window,
    }
}

fn build_mini_chart(
    id: String,
    window: Arc<str>,
    label: &str,
    values: &[f32],
    unit: Option<&str>,
    accent: AccentColor,
) -> ElementDecl {
    ElementDecl {
        id,
        kind: ElementKind::MiniChart,
        label: label.to_string(),
        value: Value::ChartValue {
            values: values.to_vec(),
            current: values.last().copied(),
            unit: unit.map(|s| s.to_string()),
        },
        meta: ElementMeta {
            accent: Some(accent.as_str().to_string()),
            ..Default::default()
        },
        window,
    }
}

fn build_plot(
    id: String,
    window: Arc<str>,
    label: &str,
    series: &[(&str, &[f32], AccentColor)],
    x_label: Option<&str>,
    y_label: Option<&str>,
) -> ElementDecl {
    let plot_series: Vec<PlotSeries> = series
        .iter()
        .map(|(name, values, color)| PlotSeries {
            name: name.to_string(),
            values: values.to_vec(),
            color: color.as_str().to_string(),
        })
        .collect();

    ElementDecl {
        id,
        kind: ElementKind::Plot,
        label: label.to_string(),
        value: Value::PlotValue {
            series: plot_series,
            x_label: x_label.map(|s| s.to_string()),
            y_label: y_label.map(|s| s.to_string()),
        },
        meta: ElementMeta::default(),
        window,
    }
}

// ── Window ───────────────────────────────────────────────────────────

/// A named window containing UI elements. Created via `Context::window()`.
pub struct Window<'a> {
    name: Arc<str>,
    ctx: &'a mut Context,
}

impl<'a> Window<'a> {
    pub(crate) fn new(name: String, ctx: &'a mut Context) -> Self {
        Self {
            name: Arc::from(name.as_str()),
            ctx,
        }
    }

    fn make_id(&self, label: &str) -> String {
        format!("{}::{}", self.name, label)
    }

    /// A floating-point slider with a range.
    pub fn slider(&mut self, label: &str, value: &mut f32, range: RangeInclusive<f32>) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::Float(v)) = self.ctx.consume_edit(&id) {
            let new = v as f32;
            let changed = *value != new;
            *value = new;
            (true, changed)
        } else {
            (false, false)
        };
        self.ctx
            .declare(build_slider(id, self.name.clone(), label, *value, &range));
        Response { clicked, changed }
    }

    /// An integer slider with a range.
    pub fn slider_int(&mut self, label: &str, value: &mut i32, range: RangeInclusive<i32>) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::Int(v)) = self.ctx.consume_edit(&id) {
            let new = v as i32;
            let changed = *value != new;
            *value = new;
            (true, changed)
        } else {
            (false, false)
        };
        self.ctx
            .declare(build_slider_int(id, self.name.clone(), label, *value, &range));
        Response { clicked, changed }
    }

    /// A checkbox (boolean toggle).
    pub fn checkbox(&mut self, label: &str, value: &mut bool) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::Bool(v)) = self.ctx.consume_edit(&id) {
            let changed = *value != v;
            *value = v;
            (true, changed)
        } else {
            (false, false)
        };
        self.ctx
            .declare(build_checkbox(id, self.name.clone(), label, *value));
        Response { clicked, changed }
    }

    /// An RGB color picker (3-component).
    pub fn color_picker(&mut self, label: &str, value: &mut [f32; 3]) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::Color3(c)) = self.ctx.consume_edit(&id) {
            let changed = *value != c;
            *value = c;
            (true, changed)
        } else {
            (false, false)
        };
        self.ctx
            .declare(build_color3(id, self.name.clone(), label, *value));
        Response { clicked, changed }
    }

    /// An RGBA color picker (4-component).
    pub fn color_picker4(&mut self, label: &str, value: &mut [f32; 4]) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::Color4(c)) = self.ctx.consume_edit(&id) {
            let changed = *value != c;
            *value = c;
            (true, changed)
        } else {
            (false, false)
        };
        self.ctx
            .declare(build_color4(id, self.name.clone(), label, *value));
        Response { clicked, changed }
    }

    /// A text input field.
    pub fn text_input(&mut self, label: &str, value: &mut String) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::String(s)) = self.ctx.consume_edit(&id) {
            let changed = *value != s;
            *value = s;
            (true, changed)
        } else {
            (false, false)
        };
        self.ctx
            .declare(build_text_input(id, self.name.clone(), label, value));
        Response { clicked, changed }
    }

    /// A dropdown selector.
    pub fn dropdown(&mut self, label: &str, selected: &mut usize, options: &[&str]) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::Enum { selected: s, .. }) = self.ctx.consume_edit(&id) {
            let changed = *selected != s;
            *selected = s;
            (true, changed)
        } else {
            (false, false)
        };
        self.ctx
            .declare(build_dropdown(id, self.name.clone(), label, *selected, options));
        Response { clicked, changed }
    }

    /// A button. Returns a `Response` — use `.clicked()` to check if it was pressed.
    pub fn button(&mut self, label: &str) -> Response {
        let id = self.make_id(label);
        let clicked = matches!(self.ctx.consume_edit(&id), Some(Value::Button(true)));
        self.ctx
            .declare(build_button(id, self.name.clone(), label));
        Response { clicked, changed: clicked }
    }

    /// A read-only text label.
    pub fn label(&mut self, text: &str) {
        let id = self.make_id(text);
        self.ctx
            .declare(build_label(id, self.name.clone(), text));
    }

    /// A visual separator line.
    pub fn separator(&mut self) {
        let id = format!("{}::__sep_{}", self.name, self.ctx.current_frame_len());
        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Separator,
            label: String::new(),
            value: Value::Bool(false),
            meta: ElementMeta::default(),
            window: self.name.clone(),
        });
    }

    /// A section header for grouping widgets.
    pub fn section(&mut self, title: &str) {
        let id = format!("{}::__sec_{}", self.name, self.ctx.current_frame_len());
        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Section,
            label: title.to_string(),
            value: Value::String(title.to_string()),
            meta: ElementMeta::default(),
            window: self.name.clone(),
        });
    }

    /// A progress bar (value from 0.0 to 1.0).
    pub fn progress_bar(&mut self, label: &str, value: f64, accent: AccentColor) {
        let id = self.make_id(label);
        self.ctx
            .declare(build_progress_bar(id, self.name.clone(), label, value, accent, None));
    }

    /// A progress bar with subtitle text.
    pub fn progress_bar_with_subtitle(
        &mut self,
        label: &str,
        value: f64,
        accent: AccentColor,
        subtitle: &str,
    ) {
        let id = self.make_id(label);
        self.ctx.declare(build_progress_bar(
            id,
            self.name.clone(),
            label,
            value,
            accent,
            Some(subtitle),
        ));
    }

    /// A stat card displaying a value with optional subvalue.
    pub fn stat(&mut self, label: &str, value: &str, subvalue: Option<&str>, accent: AccentColor) {
        let id = self.make_id(label);
        self.ctx
            .declare(build_stat(id, self.name.clone(), label, value, subvalue, accent));
    }

    /// A status indicator with colored dot.
    pub fn status(
        &mut self,
        label: &str,
        active: bool,
        active_text: Option<&str>,
        inactive_text: Option<&str>,
        active_color: AccentColor,
        inactive_color: AccentColor,
    ) {
        let id = self.make_id(label);
        self.ctx.declare(build_status(
            id,
            self.name.clone(),
            label,
            active,
            active_text,
            inactive_text,
            active_color,
            inactive_color,
        ));
    }

    /// A mini sparkline chart.
    pub fn mini_chart(
        &mut self,
        label: &str,
        values: &[f32],
        unit: Option<&str>,
        accent: AccentColor,
    ) {
        let id = self.make_id(label);
        self.ctx
            .declare(build_mini_chart(id, self.name.clone(), label, values, unit, accent));
    }

    /// Set the accent color for this window.
    /// Call this first before other widgets in the window.
    pub fn set_accent(&mut self, accent: AccentColor) {
        let id = format!("{}::__accent_{}", self.name, accent.as_str());
        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Label,
            label: String::new(),
            value: Value::String(String::new()),
            meta: ElementMeta {
                accent: Some(accent.as_str().to_string()),
                ..Default::default()
            },
            window: self.name.clone(),
        });
    }

    /// Create a grid layout container. Elements added within the closure will be
    /// arranged in a grid with the specified number of columns.
    pub fn grid<F>(&mut self, cols: usize, f: F)
    where
        F: FnOnce(&mut Grid<'_, 'a>),
    {
        let grid_id = format!("{}::__grid_{}", self.name, self.ctx.current_frame_len());
        let mut grid = Grid::new(&grid_id, self, cols);
        f(&mut grid);
        grid.finish();
    }

    /// Plot a data series as a larger chart.
    pub fn plot(
        &mut self,
        label: &str,
        series: &[(&str, &[f32], AccentColor)],
        x_label: Option<&str>,
        y_label: Option<&str>,
    ) {
        let id = self.make_id(label);
        self.ctx
            .declare(build_plot(id, self.name.clone(), label, series, x_label, y_label));
    }
}

// ── Grid ─────────────────────────────────────────────────────────────

/// A grid container for arranging elements in columns.
pub struct Grid<'a, 'ctx> {
    id: String,
    window: &'a mut Window<'ctx>,
    cols: usize,
    children: Vec<String>,
}

impl<'a, 'ctx> Grid<'a, 'ctx> {
    fn new(id: &str, window: &'a mut Window<'ctx>, cols: usize) -> Self {
        Self {
            id: id.to_string(),
            window,
            cols,
            children: Vec::new(),
        }
    }

    fn record_child(&mut self, id: String) {
        self.children.push(id);
    }

    fn finish(self) {
        self.window.ctx.declare(ElementDecl {
            id: self.id,
            kind: ElementKind::Grid,
            label: String::new(),
            value: Value::GridValue {
                cols: self.cols,
                children: self.children,
            },
            meta: ElementMeta::default(),
            window: self.window.name.clone(),
        });
    }

    fn make_id(&self, label: &str) -> String {
        format!("{}::{}", self.id, label)
    }

    // ── Interactive widgets ──────────────────────────────────────────

    /// A floating-point slider.
    pub fn slider(&mut self, label: &str, value: &mut f32, range: RangeInclusive<f32>) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::Float(v)) = self.window.ctx.consume_edit(&id) {
            let new = v as f32;
            let changed = *value != new;
            *value = new;
            (true, changed)
        } else {
            (false, false)
        };
        self.record_child(id.clone());
        self.window
            .ctx
            .declare(build_slider(id, self.window.name.clone(), label, *value, &range));
        Response { clicked, changed }
    }

    /// An integer slider.
    pub fn slider_int(&mut self, label: &str, value: &mut i32, range: RangeInclusive<i32>) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::Int(v)) = self.window.ctx.consume_edit(&id) {
            let new = v as i32;
            let changed = *value != new;
            *value = new;
            (true, changed)
        } else {
            (false, false)
        };
        self.record_child(id.clone());
        self.window.ctx.declare(build_slider_int(
            id,
            self.window.name.clone(),
            label,
            *value,
            &range,
        ));
        Response { clicked, changed }
    }

    /// A checkbox.
    pub fn checkbox(&mut self, label: &str, value: &mut bool) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::Bool(v)) = self.window.ctx.consume_edit(&id) {
            let changed = *value != v;
            *value = v;
            (true, changed)
        } else {
            (false, false)
        };
        self.record_child(id.clone());
        self.window
            .ctx
            .declare(build_checkbox(id, self.window.name.clone(), label, *value));
        Response { clicked, changed }
    }

    /// An RGB color picker.
    pub fn color_picker(&mut self, label: &str, value: &mut [f32; 3]) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::Color3(c)) = self.window.ctx.consume_edit(&id) {
            let changed = *value != c;
            *value = c;
            (true, changed)
        } else {
            (false, false)
        };
        self.record_child(id.clone());
        self.window
            .ctx
            .declare(build_color3(id, self.window.name.clone(), label, *value));
        Response { clicked, changed }
    }

    /// An RGBA color picker.
    pub fn color_picker4(&mut self, label: &str, value: &mut [f32; 4]) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::Color4(c)) = self.window.ctx.consume_edit(&id) {
            let changed = *value != c;
            *value = c;
            (true, changed)
        } else {
            (false, false)
        };
        self.record_child(id.clone());
        self.window
            .ctx
            .declare(build_color4(id, self.window.name.clone(), label, *value));
        Response { clicked, changed }
    }

    /// A text input field.
    pub fn text_input(&mut self, label: &str, value: &mut String) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::String(s)) = self.window.ctx.consume_edit(&id) {
            let changed = *value != s;
            *value = s;
            (true, changed)
        } else {
            (false, false)
        };
        self.record_child(id.clone());
        self.window
            .ctx
            .declare(build_text_input(id, self.window.name.clone(), label, value));
        Response { clicked, changed }
    }

    /// A dropdown selector.
    pub fn dropdown(&mut self, label: &str, selected: &mut usize, options: &[&str]) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::Enum { selected: s, .. }) = self.window.ctx.consume_edit(&id) {
            let changed = *selected != s;
            *selected = s;
            (true, changed)
        } else {
            (false, false)
        };
        self.record_child(id.clone());
        self.window.ctx.declare(build_dropdown(
            id,
            self.window.name.clone(),
            label,
            *selected,
            options,
        ));
        Response { clicked, changed }
    }

    /// A button. Returns a `Response` — use `.clicked()` to check if it was pressed.
    pub fn button(&mut self, label: &str) -> Response {
        let id = self.make_id(label);
        let clicked = matches!(self.window.ctx.consume_edit(&id), Some(Value::Button(true)));
        self.record_child(id.clone());
        self.window
            .ctx
            .declare(build_button(id, self.window.name.clone(), label));
        Response { clicked, changed: clicked }
    }

    // ── Display widgets ──────────────────────────────────────────────

    /// A read-only text label.
    pub fn label(&mut self, text: &str) {
        let id = self.make_id(text);
        self.record_child(id.clone());
        self.window
            .ctx
            .declare(build_label(id, self.window.name.clone(), text));
    }

    /// A progress bar (value from 0.0 to 1.0).
    pub fn progress_bar(&mut self, label: &str, value: f64, accent: AccentColor) {
        let id = self.make_id(label);
        self.record_child(id.clone());
        self.window
            .ctx
            .declare(build_progress_bar(id, self.window.name.clone(), label, value, accent, None));
    }

    /// A progress bar with subtitle text.
    pub fn progress_bar_with_subtitle(
        &mut self,
        label: &str,
        value: f64,
        accent: AccentColor,
        subtitle: &str,
    ) {
        let id = self.make_id(label);
        self.record_child(id.clone());
        self.window.ctx.declare(build_progress_bar(
            id,
            self.window.name.clone(),
            label,
            value,
            accent,
            Some(subtitle),
        ));
    }

    /// A stat card.
    pub fn stat(&mut self, label: &str, value: &str, subvalue: Option<&str>, accent: AccentColor) {
        let id = self.make_id(label);
        self.record_child(id.clone());
        self.window
            .ctx
            .declare(build_stat(id, self.window.name.clone(), label, value, subvalue, accent));
    }

    /// A status indicator.
    pub fn status(
        &mut self,
        label: &str,
        active: bool,
        active_text: Option<&str>,
        inactive_text: Option<&str>,
        active_color: AccentColor,
        inactive_color: AccentColor,
    ) {
        let id = self.make_id(label);
        self.record_child(id.clone());
        self.window.ctx.declare(build_status(
            id,
            self.window.name.clone(),
            label,
            active,
            active_text,
            inactive_text,
            active_color,
            inactive_color,
        ));
    }

    /// A mini sparkline chart.
    pub fn mini_chart(
        &mut self,
        label: &str,
        values: &[f32],
        unit: Option<&str>,
        accent: AccentColor,
    ) {
        let id = self.make_id(label);
        self.record_child(id.clone());
        self.window.ctx.declare(build_mini_chart(
            id,
            self.window.name.clone(),
            label,
            values,
            unit,
            accent,
        ));
    }

    /// A larger chart.
    pub fn plot(
        &mut self,
        label: &str,
        series: &[(&str, &[f32], AccentColor)],
        x_label: Option<&str>,
        y_label: Option<&str>,
    ) {
        let id = self.make_id(label);
        self.record_child(id.clone());
        self.window.ctx.declare(build_plot(
            id,
            self.window.name.clone(),
            label,
            series,
            x_label,
            y_label,
        ));
    }

    /// A visual separator line.
    pub fn separator(&mut self) {
        let id = format!("{}::__sep_{}", self.id, self.window.ctx.current_frame_len());
        self.record_child(id.clone());
        self.window.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Separator,
            label: String::new(),
            value: Value::Bool(false),
            meta: ElementMeta::default(),
            window: self.window.name.clone(),
        });
    }

    /// Create a nested grid layout container.
    pub fn grid<F>(&mut self, cols: usize, f: F)
    where
        F: FnOnce(&mut Grid<'_, 'ctx>),
    {
        let grid_id = format!("{}::__grid_{}", self.id, self.window.ctx.current_frame_len());
        let mut child_grid = Grid::new(&grid_id, self.window, cols);
        f(&mut child_grid);
        let child_id = child_grid.id.clone();
        child_grid.finish();
        self.children.push(child_id);
    }
}
