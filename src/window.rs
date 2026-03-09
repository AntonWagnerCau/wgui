use std::ops::RangeInclusive;

use crate::context::Context;
use crate::element::{AccentColor, ElementDecl, ElementKind, ElementMeta, PlotSeries, Value};

/// A named window containing UI elements. Created via `Context::window()`.
pub struct Window<'a> {
    name: String,
    ctx: &'a mut Context,
}

impl<'a> Window<'a> {
    pub(crate) fn new(name: String, ctx: &'a mut Context) -> Self {
        Self { name, ctx }
    }

    fn make_id(&self, label: &str) -> String {
        format!("{}::{}", self.name, label)
    }

    /// A floating-point slider with a range.
    pub fn slider(&mut self, label: &str, value: &mut f32, range: RangeInclusive<f32>) {
        let id = self.make_id(label);

        if let Some(Value::Float(v)) = self.ctx.consume_edit(&id) {
            *value = v as f32;
        }

        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Slider,
            label: label.to_string(),
            value: Value::Float(*value as f64),
            meta: ElementMeta {
                min: Some(*range.start() as f64),
                max: Some(*range.end() as f64),
                step: Some(0.01),
                ..Default::default()
            },
            window: self.name.clone(),
        });
    }

    /// An integer slider with a range.
    pub fn slider_int(&mut self, label: &str, value: &mut i32, range: RangeInclusive<i32>) {
        let id = self.make_id(label);

        if let Some(Value::Int(v)) = self.ctx.consume_edit(&id) {
            *value = v as i32;
        }

        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Slider,
            label: label.to_string(),
            value: Value::Int(*value as i64),
            meta: ElementMeta {
                min: Some(*range.start() as f64),
                max: Some(*range.end() as f64),
                step: Some(1.0),
                ..Default::default()
            },
            window: self.name.clone(),
        });
    }

    /// A checkbox (boolean toggle).
    pub fn checkbox(&mut self, label: &str, value: &mut bool) {
        let id = self.make_id(label);

        if let Some(Value::Bool(v)) = self.ctx.consume_edit(&id) {
            *value = v;
        }

        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Checkbox,
            label: label.to_string(),
            value: Value::Bool(*value),
            meta: ElementMeta::default(),
            window: self.name.clone(),
        });
    }

    /// An RGB color picker (3-component).
    pub fn color_picker(&mut self, label: &str, value: &mut [f32; 3]) {
        let id = self.make_id(label);

        if let Some(Value::Color3(c)) = self.ctx.consume_edit(&id) {
            *value = c;
        }

        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::ColorPicker3,
            label: label.to_string(),
            value: Value::Color3(*value),
            meta: ElementMeta::default(),
            window: self.name.clone(),
        });
    }

    /// An RGBA color picker (4-component).
    pub fn color_picker4(&mut self, label: &str, value: &mut [f32; 4]) {
        let id = self.make_id(label);

        if let Some(Value::Color4(c)) = self.ctx.consume_edit(&id) {
            *value = c;
        }

        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::ColorPicker4,
            label: label.to_string(),
            value: Value::Color4(*value),
            meta: ElementMeta::default(),
            window: self.name.clone(),
        });
    }

    /// A text input field.
    pub fn text_input(&mut self, label: &str, value: &mut String) {
        let id = self.make_id(label);

        if let Some(Value::String(s)) = self.ctx.consume_edit(&id) {
            *value = s;
        }

        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::TextInput,
            label: label.to_string(),
            value: Value::String(value.clone()),
            meta: ElementMeta::default(),
            window: self.name.clone(),
        });
    }

    /// A dropdown selector. Returns the selected index.
    pub fn dropdown(&mut self, label: &str, selected: &mut usize, options: &[&str]) {
        let id = self.make_id(label);

        if let Some(Value::Enum {
            selected: s,
            options: _,
        }) = self.ctx.consume_edit(&id)
        {
            *selected = s;
        }

        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Dropdown,
            label: label.to_string(),
            value: Value::Enum {
                selected: *selected,
                options: options.iter().map(|s| s.to_string()).collect(),
            },
            meta: ElementMeta::default(),
            window: self.name.clone(),
        });
    }

    /// A button. Returns `true` for one frame when clicked.
    pub fn button(&mut self, label: &str) -> bool {
        let id = self.make_id(label);

        let clicked = matches!(self.ctx.consume_edit(&id), Some(Value::Button(true)));

        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Button,
            label: label.to_string(),
            value: Value::Button(false),
            meta: ElementMeta::default(),
            window: self.name.clone(),
        });

        clicked
    }

    /// A read-only text label.
    pub fn label(&mut self, text: &str) {
        let id = self.make_id(text);

        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Label,
            label: text.to_string(),
            value: Value::String(text.to_string()),
            meta: ElementMeta::default(),
            window: self.name.clone(),
        });
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

    /// A progress bar (0.0 to 1.0 or 0 to 100).
    /// 
    /// # Example
    /// ```
    /// let mut progress = 0.75;
    /// win.progress_bar("Loading", progress, AccentColor::Blue);
    /// ```
    pub fn progress_bar(&mut self, label: &str, value: f64, accent: AccentColor) {
        let id = self.make_id(label);
        let clamped = value.clamp(0.0, 1.0);

        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::ProgressBar,
            label: label.to_string(),
            value: Value::Progress(clamped),
            meta: ElementMeta {
                accent: Some(accent.as_str().to_string()),
                ..Default::default()
            },
            window: self.name.clone(),
        });
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
        let clamped = value.clamp(0.0, 1.0);

        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::ProgressBar,
            label: label.to_string(),
            value: Value::Progress(clamped),
            meta: ElementMeta {
                accent: Some(accent.as_str().to_string()),
                subtitle: Some(subtitle.to_string()),
                ..Default::default()
            },
            window: self.name.clone(),
        });
    }

    /// A stat card displaying a value with optional subvalue.
    ///
    /// # Example
    /// ```
    /// win.stat("FPS", "60", Some("avg 58"), AccentColor::Green);
    /// ```
    pub fn stat(&mut self, label: &str, value: &str, subvalue: Option<&str>, accent: AccentColor) {
        let id = self.make_id(label);

        self.ctx.declare(ElementDecl {
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
            window: self.name.clone(),
        });
    }

    /// A status indicator with colored dot.
    ///
    /// # Example
    /// ```
    /// win.status("AI State", true, Some("Enabled"), Some("Disabled"), AccentColor::Green, AccentColor::Red);
    /// ```
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

        self.ctx.declare(ElementDecl {
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
            window: self.name.clone(),
        });
    }

    /// A mini sparkline chart.
    ///
    /// # Example
    /// ```
    /// let values = vec![10.0, 15.0, 12.0, 18.0, 20.0];
    /// win.mini_chart("Velocity", &values, Some("m/s"), AccentColor::Coral);
    /// ```
    pub fn mini_chart(&mut self, label: &str, values: &[f32], unit: Option<&str>, accent: AccentColor) {
        let id = self.make_id(label);
        let current = values.last().copied();

        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::MiniChart,
            label: label.to_string(),
            value: Value::ChartValue {
                values: values.to_vec(),
                current,
                unit: unit.map(|s| s.to_string()),
            },
            meta: ElementMeta {
                accent: Some(accent.as_str().to_string()),
                ..Default::default()
            },
            window: self.name.clone(),
        });
    }

    /// Set the accent color for this window (affects all cards in the window).
    /// Call this first before other widgets in the window.
    pub fn set_accent(&mut self, accent: AccentColor) {
        // This is a marker element that sets the window's accent color
        // The frontend uses the accent of the first element
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
    ///
    /// # Example
    /// ```
    /// win.grid(2, |grid| {
    ///     grid.stat("FPS", "60", None, AccentColor::Green);
    ///     grid.stat("MS", "16.7", Some("avg"), AccentColor::Blue);
    ///     grid.stat("Entities", "1024", None, AccentColor::Coral);
    ///     grid.stat("Memory", "256", Some("MB"), AccentColor::Purple);
    /// });
    /// ```
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
    ///
    /// # Example
    /// ```
    /// let values = vec![10.0, 15.0, 12.0, 18.0, 20.0, 22.0, 19.0];
    /// win.plot("Performance", &[("FPS", &values, AccentColor::Green)], Some("Time"), Some("FPS"));
    /// ```
    pub fn plot(
        &mut self,
        label: &str,
        series: &[(&str, &[f32], AccentColor)],
        x_label: Option<&str>,
        y_label: Option<&str>,
    ) {
        let id = self.make_id(label);

        let plot_series: Vec<PlotSeries> = series
            .iter()
            .map(|(name, values, color)| PlotSeries {
                name: name.to_string(),
                values: values.to_vec(),
                color: color.as_str().to_string(),
            })
            .collect();

        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Plot,
            label: label.to_string(),
            value: Value::PlotValue {
                series: plot_series,
                x_label: x_label.map(|s| s.to_string()),
                y_label: y_label.map(|s| s.to_string()),
            },
            meta: ElementMeta::default(),
            window: self.name.clone(),
        });
    }
}

/// A grid container for grouping elements.
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
        // Declare the grid container with references to all children
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

    /// Add a stat card to the grid.
    pub fn stat(&mut self, label: &str, value: &str, subvalue: Option<&str>, accent: AccentColor) {
        let id = self.make_id(label);

        self.record_child(id.clone());
        self.window.ctx.declare(ElementDecl {
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
            window: self.window.name.clone(),
        });
    }

    /// Add a mini chart to the grid.
    pub fn mini_chart(&mut self, label: &str, values: &[f32], unit: Option<&str>, accent: AccentColor) {
        let id = self.make_id(label);
        let current = values.last().copied();

        self.record_child(id.clone());
        self.window.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::MiniChart,
            label: label.to_string(),
            value: Value::ChartValue {
                values: values.to_vec(),
                current,
                unit: unit.map(|s| s.to_string()),
            },
            meta: ElementMeta {
                accent: Some(accent.as_str().to_string()),
                ..Default::default()
            },
            window: self.window.name.clone(),
        });
    }

    /// Add a progress bar to the grid.
    pub fn progress_bar(&mut self, label: &str, value: f64, accent: AccentColor) {
        let id = self.make_id(label);
        let clamped = value.clamp(0.0, 1.0);

        self.record_child(id.clone());
        self.window.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::ProgressBar,
            label: label.to_string(),
            value: Value::Progress(clamped),
            meta: ElementMeta {
                accent: Some(accent.as_str().to_string()),
                ..Default::default()
            },
            window: self.window.name.clone(),
        });
    }

    /// Add a status indicator to the grid.
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
        self.window.ctx.declare(ElementDecl {
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
            window: self.window.name.clone(),
        });
    }
}
