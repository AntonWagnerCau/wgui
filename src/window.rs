use std::collections::HashMap;
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

// ── WidgetSink trait ─────────────────────────────────────────────────
// Shared interface for Window, Grid, and Horizontal so widget functions
// can be written once and called from any container.

pub(crate) trait WidgetSink {
    fn make_id(&mut self, label: &str) -> String;
    fn declare(&mut self, decl: ElementDecl);
    fn consume_edit(&mut self, id: &str) -> Option<Value>;
    fn window_name(&self) -> Arc<str>;
    fn record_child(&mut self, id: String);
}

// ── Generic widget functions ─────────────────────────────────────────

fn widget_slider(sink: &mut impl WidgetSink, label: &str, value: &mut f32, range: &RangeInclusive<f32>) -> Response {
    let id = sink.make_id(label);
    let (clicked, changed) = if let Some(Value::Float(v)) = sink.consume_edit(&id) {
        let new = v as f32;
        let changed = *value != new;
        *value = new;
        (true, changed)
    } else {
        (false, false)
    };
    sink.record_child(id.clone());
    let step = (*range.end() as f64 - *range.start() as f64) / 10000.0;
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::Slider,
        label: label.to_string(),
        value: Value::Float(*value as f64),
        meta: ElementMeta {
            min: Some(*range.start() as f64),
            max: Some(*range.end() as f64),
            step: Some(step),
            ..Default::default()
        },
        window: sink.window_name(),
    });
    Response { clicked, changed }
}

fn widget_slider_f64(sink: &mut impl WidgetSink, label: &str, value: &mut f64, range: &RangeInclusive<f64>) -> Response {
    let id = sink.make_id(label);
    let (clicked, changed) = if let Some(Value::Float(v)) = sink.consume_edit(&id) {
        let changed = *value != v;
        *value = v;
        (true, changed)
    } else {
        (false, false)
    };
    sink.record_child(id.clone());
    let step = (*range.end() - *range.start()) / 10000.0;
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::Slider,
        label: label.to_string(),
        value: Value::Float(*value),
        meta: ElementMeta {
            min: Some(*range.start()),
            max: Some(*range.end()),
            step: Some(step),
            ..Default::default()
        },
        window: sink.window_name(),
    });
    Response { clicked, changed }
}

fn widget_slider_int(sink: &mut impl WidgetSink, label: &str, value: &mut i32, range: &RangeInclusive<i32>) -> Response {
    let id = sink.make_id(label);
    let (clicked, changed) = if let Some(Value::Int(v)) = sink.consume_edit(&id) {
        let new = v as i32;
        let changed = *value != new;
        *value = new;
        (true, changed)
    } else {
        (false, false)
    };
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
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
        window: sink.window_name(),
    });
    Response { clicked, changed }
}

fn widget_slider_uint(sink: &mut impl WidgetSink, label: &str, value: &mut u32, range: &RangeInclusive<u32>) -> Response {
    let id = sink.make_id(label);
    let (clicked, changed) = if let Some(Value::Int(v)) = sink.consume_edit(&id) {
        let new = v as u32;
        let changed = *value != new;
        *value = new;
        (true, changed)
    } else {
        (false, false)
    };
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
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
        window: sink.window_name(),
    });
    Response { clicked, changed }
}

fn widget_checkbox(sink: &mut impl WidgetSink, label: &str, value: &mut bool) -> Response {
    let id = sink.make_id(label);
    let (clicked, changed) = if let Some(Value::Bool(v)) = sink.consume_edit(&id) {
        let changed = *value != v;
        *value = v;
        (true, changed)
    } else {
        (false, false)
    };
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::Checkbox,
        label: label.to_string(),
        value: Value::Bool(*value),
        meta: ElementMeta::default(),
        window: sink.window_name(),
    });
    Response { clicked, changed }
}

fn widget_color3(sink: &mut impl WidgetSink, label: &str, value: &mut [f32; 3]) -> Response {
    let id = sink.make_id(label);
    let (clicked, changed) = if let Some(Value::Color3(c)) = sink.consume_edit(&id) {
        let changed = *value != c;
        *value = c;
        (true, changed)
    } else {
        (false, false)
    };
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::ColorPicker3,
        label: label.to_string(),
        value: Value::Color3(*value),
        meta: ElementMeta::default(),
        window: sink.window_name(),
    });
    Response { clicked, changed }
}

fn widget_color4(sink: &mut impl WidgetSink, label: &str, value: &mut [f32; 4]) -> Response {
    let id = sink.make_id(label);
    let (clicked, changed) = if let Some(Value::Color4(c)) = sink.consume_edit(&id) {
        let changed = *value != c;
        *value = c;
        (true, changed)
    } else {
        (false, false)
    };
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::ColorPicker4,
        label: label.to_string(),
        value: Value::Color4(*value),
        meta: ElementMeta::default(),
        window: sink.window_name(),
    });
    Response { clicked, changed }
}

fn widget_text_input(sink: &mut impl WidgetSink, label: &str, value: &mut String) -> Response {
    let id = sink.make_id(label);
    let (clicked, changed) = if let Some(Value::String(s)) = sink.consume_edit(&id) {
        let changed = *value != s;
        *value = s;
        (true, changed)
    } else {
        (false, false)
    };
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::TextInput,
        label: label.to_string(),
        value: Value::String(value.clone()),
        meta: ElementMeta::default(),
        window: sink.window_name(),
    });
    Response { clicked, changed }
}

fn widget_dropdown(sink: &mut impl WidgetSink, label: &str, selected: &mut usize, options: &[&str]) -> Response {
    let id = sink.make_id(label);
    let (clicked, changed) = if let Some(Value::Enum { selected: s, .. }) = sink.consume_edit(&id) {
        let changed = *selected != s;
        *selected = s;
        (true, changed)
    } else {
        (false, false)
    };
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::Dropdown,
        label: label.to_string(),
        value: Value::Enum {
            selected: *selected,
            options: options.iter().map(|s| s.to_string()).collect(),
        },
        meta: ElementMeta::default(),
        window: sink.window_name(),
    });
    Response { clicked, changed }
}

fn widget_button(sink: &mut impl WidgetSink, label: &str) -> Response {
    let id = sink.make_id(label);
    let clicked = matches!(sink.consume_edit(&id), Some(Value::Button(true)));
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::Button,
        label: label.to_string(),
        value: Value::Button(false),
        meta: ElementMeta::default(),
        window: sink.window_name(),
    });
    Response { clicked, changed: clicked }
}

fn widget_button_compact(sink: &mut impl WidgetSink, label: &str, accent: Option<AccentColor>) -> Response {
    let id = sink.make_id(label);
    let clicked = matches!(sink.consume_edit(&id), Some(Value::Button(true)));
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::ButtonCompact,
        label: label.to_string(),
        value: Value::Button(false),
        meta: ElementMeta {
            accent,
            ..Default::default()
        },
        window: sink.window_name(),
    });
    Response { clicked, changed: clicked }
}

fn widget_label(sink: &mut impl WidgetSink, text: &str) {
    let id = sink.make_id("__label");
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::Label,
        label: String::new(),
        value: Value::String(text.to_string()),
        meta: ElementMeta::default(),
        window: sink.window_name(),
    });
}

fn widget_kv(sink: &mut impl WidgetSink, label: &str, value: &str) {
    let id = sink.make_id(label);
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::KeyValue,
        label: label.to_string(),
        value: Value::String(value.to_string()),
        meta: ElementMeta::default(),
        window: sink.window_name(),
    });
}

fn widget_progress_bar(sink: &mut impl WidgetSink, label: &str, value: f64, accent: AccentColor, subtitle: Option<&str>) {
    let id = sink.make_id(label);
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::ProgressBar,
        label: label.to_string(),
        value: Value::Progress(value.clamp(0.0, 1.0)),
        meta: ElementMeta {
            accent: Some(accent),
            subtitle: subtitle.map(|s| s.to_string()),
            ..Default::default()
        },
        window: sink.window_name(),
    });
}

fn widget_stat(sink: &mut impl WidgetSink, label: &str, value: &str, subvalue: Option<&str>, accent: AccentColor) {
    let id = sink.make_id(label);
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::Stat,
        label: label.to_string(),
        value: Value::StatValue {
            value: value.to_string(),
            subvalue: subvalue.map(|s| s.to_string()),
        },
        meta: ElementMeta {
            accent: Some(accent),
            ..Default::default()
        },
        window: sink.window_name(),
    });
}

fn widget_status(
    sink: &mut impl WidgetSink,
    label: &str,
    active: bool,
    active_text: Option<&str>,
    inactive_text: Option<&str>,
    active_color: AccentColor,
    inactive_color: AccentColor,
) {
    let id = sink.make_id(label);
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
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
        window: sink.window_name(),
    });
}

fn widget_mini_chart(sink: &mut impl WidgetSink, label: &str, values: &[f32], unit: Option<&str>, accent: AccentColor) {
    let id = sink.make_id(label);
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::MiniChart,
        label: label.to_string(),
        value: Value::ChartValue {
            values: values.to_vec(),
            current: values.last().copied(),
            unit: unit.map(|s| s.to_string()),
        },
        meta: ElementMeta {
            accent: Some(accent),
            ..Default::default()
        },
        window: sink.window_name(),
    });
}

fn widget_plot(
    sink: &mut impl WidgetSink,
    label: &str,
    series: &[(&str, &[f32], AccentColor, bool)],
    x_label: Option<&str>,
    y_label: Option<&str>,
) {
    let id = sink.make_id(label);
    let plot_series: Vec<PlotSeries> = series
        .iter()
        .map(|(name, values, color, autoscale)| PlotSeries {
            name: name.to_string(),
            values: values.to_vec(),
            color: color.as_str().to_string(),
            autoscale: *autoscale,
        })
        .collect();
    sink.record_child(id.clone());
    sink.declare(ElementDecl {
        id,
        kind: ElementKind::Plot,
        label: label.to_string(),
        value: Value::PlotValue {
            series: plot_series,
            x_label: x_label.map(|s| s.to_string()),
            y_label: y_label.map(|s| s.to_string()),
        },
        meta: ElementMeta::default(),
        window: sink.window_name(),
    });
}

// ── Label-based ID generation ────────────────────────────────────────

fn make_label_id(prefix: &str, label: &str, label_counts: &mut HashMap<String, usize>) -> String {
    let count = label_counts.entry(label.to_string()).or_insert(0);
    let id = if *count == 0 {
        format!("{prefix}::{label}")
    } else {
        format!("{prefix}::{label}#{count}")
    };
    *count += 1;
    id
}

// ── Window ─────────────────────────────────────────────────────────--

/// A named window containing UI elements. Created via `Context::window()`.
pub struct Window<'a> {
    name: Arc<str>,
    ctx: &'a mut Context,
    label_counts: HashMap<String, usize>,
}

impl<'a> WidgetSink for Window<'a> {
    fn make_id(&mut self, label: &str) -> String {
        make_label_id(&self.name, label, &mut self.label_counts)
    }

    fn declare(&mut self, decl: ElementDecl) {
        self.ctx.declare(decl);
    }

    fn consume_edit(&mut self, id: &str) -> Option<Value> {
        self.ctx.consume_edit(id)
    }

    fn window_name(&self) -> Arc<str> {
        self.name.clone()
    }

    fn record_child(&mut self, _id: String) {
        // Window is top-level — no parent to record into
    }
}

impl<'a> Window<'a> {
    pub(crate) fn new(name: String, ctx: &'a mut Context) -> Self {
        Self {
            name: Arc::from(name.as_str()),
            ctx,
            label_counts: HashMap::new(),
        }
    }

    pub fn slider(&mut self, label: &str, value: &mut f32, range: RangeInclusive<f32>) -> Response {
        widget_slider(self, label, value, &range)
    }

    pub fn slider_f64(&mut self, label: &str, value: &mut f64, range: RangeInclusive<f64>) -> Response {
        widget_slider_f64(self, label, value, &range)
    }

    pub fn slider_int(&mut self, label: &str, value: &mut i32, range: RangeInclusive<i32>) -> Response {
        widget_slider_int(self, label, value, &range)
    }

    pub fn slider_uint(&mut self, label: &str, value: &mut u32, range: RangeInclusive<u32>) -> Response {
        widget_slider_uint(self, label, value, &range)
    }

    pub fn checkbox(&mut self, label: &str, value: &mut bool) -> Response {
        widget_checkbox(self, label, value)
    }

    pub fn color_picker(&mut self, label: &str, value: &mut [f32; 3]) -> Response {
        widget_color3(self, label, value)
    }

    pub fn color_picker4(&mut self, label: &str, value: &mut [f32; 4]) -> Response {
        widget_color4(self, label, value)
    }

    pub fn text_input(&mut self, label: &str, value: &mut String) -> Response {
        widget_text_input(self, label, value)
    }

    pub fn dropdown(&mut self, label: &str, selected: &mut usize, options: &[&str]) -> Response {
        widget_dropdown(self, label, selected, options)
    }

    pub fn button(&mut self, label: &str) -> Response {
        widget_button(self, label)
    }

    pub fn label(&mut self, text: &str) {
        widget_label(self, text);
    }

    pub fn kv(&mut self, label: &str, value: &str) {
        widget_kv(self, label, value);
    }

    pub fn kv_value(&mut self, label: &str, value: &mut String) -> Response {
        let id = self.make_id(label);
        let (clicked, changed) = if let Some(Value::String(v)) = self.ctx.consume_edit(&id) {
            let changed = *value != v;
            *value = v;
            (true, changed)
        } else {
            (false, false)
        };
        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::KeyValue,
            label: label.to_string(),
            value: Value::String(value.clone()),
            meta: ElementMeta::default(),
            window: self.name.clone(),
        });
        Response { clicked, changed }
    }

    pub fn button_compact(&mut self, label: &str) -> Response {
        widget_button_compact(self, label, None)
    }

    pub fn button_compact_accent(&mut self, label: &str, accent: AccentColor) -> Response {
        widget_button_compact(self, label, Some(accent))
    }

    pub fn horizontal<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Horizontal<'_, 'a>),
    {
        let h_id = self.make_id("__horiz");
        let mut horiz = Horizontal::new(h_id, self);
        f(&mut horiz);
        horiz.finish();
    }

    pub fn separator(&mut self) {
        let id = self.make_id("__sep");
        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Separator,
            label: String::new(),
            value: Value::Bool(false),
            meta: ElementMeta::default(),
            window: self.name.clone(),
        });
    }

    pub fn section(&mut self, title: &str) {
        let id = self.make_id(title);
        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Section,
            label: title.to_string(),
            value: Value::String(title.to_string()),
            meta: ElementMeta::default(),
            window: self.name.clone(),
        });
    }

    pub fn progress_bar(&mut self, label: &str, value: f64, accent: AccentColor) {
        widget_progress_bar(self, label, value, accent, None);
    }

    pub fn progress_bar_with_subtitle(&mut self, label: &str, value: f64, accent: AccentColor, subtitle: &str) {
        widget_progress_bar(self, label, value, accent, Some(subtitle));
    }

    pub fn stat(&mut self, label: &str, value: &str, subvalue: Option<&str>, accent: AccentColor) {
        widget_stat(self, label, value, subvalue, accent);
    }

    pub fn status(
        &mut self,
        label: &str,
        active: bool,
        active_text: Option<&str>,
        inactive_text: Option<&str>,
        active_color: AccentColor,
        inactive_color: AccentColor,
    ) {
        widget_status(self, label, active, active_text, inactive_text, active_color, inactive_color);
    }

    pub fn mini_chart(&mut self, label: &str, values: &[f32], unit: Option<&str>, accent: AccentColor) {
        widget_mini_chart(self, label, values, unit, accent);
    }

    pub fn set_accent(&mut self, accent: AccentColor) {
        let id = self.make_id(&format!("__accent_{}", accent.as_str()));
        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Label,
            label: String::new(),
            value: Value::String(String::new()),
            meta: ElementMeta {
                accent: Some(accent),
                ..Default::default()
            },
            window: self.name.clone(),
        });
    }

    pub fn grid<F>(&mut self, cols: usize, f: F)
    where
        F: FnOnce(&mut Grid<'_, 'a>),
    {
        let grid_id = self.make_id("__grid");
        let mut grid = Grid::new(grid_id, self, cols);
        f(&mut grid);
        grid.finish();
    }

    pub fn plot(
        &mut self,
        label: &str,
        series: &[(&str, &[f32], AccentColor)],
        x_label: Option<&str>,
        y_label: Option<&str>,
    ) {
        // Default to autoscale=true for backward compatibility
        let series_with_autoscale: Vec<(&str, &[f32], AccentColor, bool)> = series
            .iter()
            .map(|(name, values, color)| (*name, *values, *color, true))
            .collect();
        widget_plot(self, label, &series_with_autoscale, x_label, y_label);
    }

    /// Plot with explicit autoscale control per series
    /// series: (name, values, color, autoscale)
    pub fn plot_with_autoscale(
        &mut self,
        label: &str,
        series: &[(&str, &[f32], AccentColor, bool)],
        x_label: Option<&str>,
        y_label: Option<&str>,
    ) {
        widget_plot(self, label, series, x_label, y_label);
    }
}

// ── Horizontal ───────────────────────────────────────────────────────

/// A horizontal layout container for arranging widgets side by side.
pub struct Horizontal<'a, 'ctx> {
    id: String,
    window: &'a mut Window<'ctx>,
    children: Vec<String>,
    label_counts: HashMap<String, usize>,
}

impl<'a, 'ctx> WidgetSink for Horizontal<'a, 'ctx> {
    fn make_id(&mut self, label: &str) -> String {
        make_label_id(&self.id, label, &mut self.label_counts)
    }

    fn declare(&mut self, decl: ElementDecl) {
        self.window.ctx.declare(decl);
    }

    fn consume_edit(&mut self, id: &str) -> Option<Value> {
        self.window.ctx.consume_edit(id)
    }

    fn window_name(&self) -> Arc<str> {
        self.window.name.clone()
    }

    fn record_child(&mut self, id: String) {
        self.children.push(id);
    }
}

impl<'a, 'ctx> Horizontal<'a, 'ctx> {
    fn new(id: String, window: &'a mut Window<'ctx>) -> Self {
        Self {
            id,
            window,
            children: Vec::new(),
            label_counts: HashMap::new(),
        }
    }

    fn finish(self) {
        self.window.ctx.declare(ElementDecl {
            id: self.id,
            kind: ElementKind::Horizontal,
            label: String::new(),
            value: Value::GridValue {
                cols: self.children.len(),
                children: self.children,
            },
            meta: ElementMeta::default(),
            window: self.window.name.clone(),
        });
    }

    pub fn button(&mut self, label: &str) -> Response {
        self.button_accent_inner(label, None)
    }

    pub fn button_accent(&mut self, label: &str, accent: AccentColor) -> Response {
        self.button_accent_inner(label, Some(accent))
    }

    fn button_accent_inner(&mut self, label: &str, accent: Option<AccentColor>) -> Response {
        let id = self.make_id(label);
        let clicked = matches!(self.window.ctx.consume_edit(&id), Some(Value::Button(true)));
        self.children.push(id.clone());
        self.window.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::ButtonInline,
            label: label.to_string(),
            value: Value::Button(false),
            meta: ElementMeta {
                accent,
                ..Default::default()
            },
            window: self.window.name.clone(),
        });
        Response { clicked, changed: clicked }
    }

    pub fn label(&mut self, text: &str) {
        widget_label(self, text);
    }

    pub fn kv(&mut self, label: &str, value: &str) {
        widget_kv(self, label, value);
    }

    pub fn text_input(&mut self, label: &str, value: &mut String) -> Response {
        widget_text_input(self, label, value)
    }

    pub fn text_input_inline(&mut self, placeholder: &str, value: &mut String) -> Response {
        let id = self.make_id(placeholder);
        let (clicked, changed) = if let Some(Value::String(s)) = self.window.ctx.consume_edit(&id) {
            let changed = *value != s;
            *value = s;
            (true, changed)
        } else {
            (false, false)
        };
        self.children.push(id.clone());
        self.window.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::TextInputInline,
            label: placeholder.to_string(),
            value: Value::String(value.clone()),
            meta: ElementMeta::default(),
            window: self.window.name.clone(),
        });
        Response { clicked, changed }
    }

    pub fn slider(&mut self, label: &str, value: &mut f32, range: RangeInclusive<f32>) -> Response {
        widget_slider(self, label, value, &range)
    }

    pub fn slider_f64(&mut self, label: &str, value: &mut f64, range: RangeInclusive<f64>) -> Response {
        widget_slider_f64(self, label, value, &range)
    }

    pub fn slider_int(&mut self, label: &str, value: &mut i32, range: RangeInclusive<i32>) -> Response {
        widget_slider_int(self, label, value, &range)
    }

    pub fn slider_uint(&mut self, label: &str, value: &mut u32, range: RangeInclusive<u32>) -> Response {
        widget_slider_uint(self, label, value, &range)
    }

    pub fn checkbox(&mut self, label: &str, value: &mut bool) -> Response {
        widget_checkbox(self, label, value)
    }

    pub fn color_picker(&mut self, label: &str, value: &mut [f32; 3]) -> Response {
        widget_color3(self, label, value)
    }

    pub fn color_picker4(&mut self, label: &str, value: &mut [f32; 4]) -> Response {
        widget_color4(self, label, value)
    }

    pub fn dropdown(&mut self, label: &str, selected: &mut usize, options: &[&str]) -> Response {
        widget_dropdown(self, label, selected, options)
    }
}

// ── Grid ─────────────────────────────────────────────────────────────

/// A grid container for arranging elements in columns.
pub struct Grid<'a, 'ctx> {
    id: String,
    window: &'a mut Window<'ctx>,
    cols: usize,
    children: Vec<String>,
    label_counts: HashMap<String, usize>,
}

impl<'a, 'ctx> WidgetSink for Grid<'a, 'ctx> {
    fn make_id(&mut self, label: &str) -> String {
        make_label_id(&self.id, label, &mut self.label_counts)
    }

    fn declare(&mut self, decl: ElementDecl) {
        self.window.ctx.declare(decl);
    }

    fn consume_edit(&mut self, id: &str) -> Option<Value> {
        self.window.ctx.consume_edit(id)
    }

    fn window_name(&self) -> Arc<str> {
        self.window.name.clone()
    }

    fn record_child(&mut self, id: String) {
        self.children.push(id);
    }
}

impl<'a, 'ctx> Grid<'a, 'ctx> {
    fn new(id: String, window: &'a mut Window<'ctx>, cols: usize) -> Self {
        Self {
            id,
            window,
            cols,
            children: Vec::new(),
            label_counts: HashMap::new(),
        }
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

    pub fn slider(&mut self, label: &str, value: &mut f32, range: RangeInclusive<f32>) -> Response {
        widget_slider(self, label, value, &range)
    }

    pub fn slider_f64(&mut self, label: &str, value: &mut f64, range: RangeInclusive<f64>) -> Response {
        widget_slider_f64(self, label, value, &range)
    }

    pub fn slider_int(&mut self, label: &str, value: &mut i32, range: RangeInclusive<i32>) -> Response {
        widget_slider_int(self, label, value, &range)
    }

    pub fn slider_uint(&mut self, label: &str, value: &mut u32, range: RangeInclusive<u32>) -> Response {
        widget_slider_uint(self, label, value, &range)
    }

    pub fn checkbox(&mut self, label: &str, value: &mut bool) -> Response {
        widget_checkbox(self, label, value)
    }

    pub fn color_picker(&mut self, label: &str, value: &mut [f32; 3]) -> Response {
        widget_color3(self, label, value)
    }

    pub fn color_picker4(&mut self, label: &str, value: &mut [f32; 4]) -> Response {
        widget_color4(self, label, value)
    }

    pub fn text_input(&mut self, label: &str, value: &mut String) -> Response {
        widget_text_input(self, label, value)
    }

    pub fn dropdown(&mut self, label: &str, selected: &mut usize, options: &[&str]) -> Response {
        widget_dropdown(self, label, selected, options)
    }

    pub fn button(&mut self, label: &str) -> Response {
        widget_button(self, label)
    }

    pub fn label(&mut self, text: &str) {
        widget_label(self, text);
    }

    pub fn progress_bar(&mut self, label: &str, value: f64, accent: AccentColor) {
        widget_progress_bar(self, label, value, accent, None);
    }

    pub fn progress_bar_with_subtitle(&mut self, label: &str, value: f64, accent: AccentColor, subtitle: &str) {
        widget_progress_bar(self, label, value, accent, Some(subtitle));
    }

    pub fn stat(&mut self, label: &str, value: &str, subvalue: Option<&str>, accent: AccentColor) {
        widget_stat(self, label, value, subvalue, accent);
    }

    pub fn status(
        &mut self,
        label: &str,
        active: bool,
        active_text: Option<&str>,
        inactive_text: Option<&str>,
        active_color: AccentColor,
        inactive_color: AccentColor,
    ) {
        widget_status(self, label, active, active_text, inactive_text, active_color, inactive_color);
    }

    pub fn mini_chart(&mut self, label: &str, values: &[f32], unit: Option<&str>, accent: AccentColor) {
        widget_mini_chart(self, label, values, unit, accent);
    }

    pub fn plot(
        &mut self,
        label: &str,
        series: &[(&str, &[f32], AccentColor)],
        x_label: Option<&str>,
        y_label: Option<&str>,
    ) {
        // Default to autoscale=true for backward compatibility
        let series_with_autoscale: Vec<(&str, &[f32], AccentColor, bool)> = series
            .iter()
            .map(|(name, values, color)| (*name, *values, *color, true))
            .collect();
        widget_plot(self, label, &series_with_autoscale, x_label, y_label);
    }

    /// Plot with explicit autoscale control per series
    /// series: (name, values, color, autoscale)
    pub fn plot_with_autoscale(
        &mut self,
        label: &str,
        series: &[(&str, &[f32], AccentColor, bool)],
        x_label: Option<&str>,
        y_label: Option<&str>,
    ) {
        widget_plot(self, label, series, x_label, y_label);
    }

    pub fn button_compact(&mut self, label: &str) -> Response {
        widget_button_compact(self, label, None)
    }

    pub fn button_compact_accent(&mut self, label: &str, accent: AccentColor) -> Response {
        widget_button_compact(self, label, Some(accent))
    }

    pub fn kv(&mut self, label: &str, value: &str) {
        widget_kv(self, label, value);
    }

    pub fn separator(&mut self) {
        let id = self.make_id("__sep");
        self.children.push(id.clone());
        self.window.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Separator,
            label: String::new(),
            value: Value::Bool(false),
            meta: ElementMeta::default(),
            window: self.window.name.clone(),
        });
    }

    pub fn grid<F>(&mut self, cols: usize, f: F)
    where
        F: FnOnce(&mut Grid<'_, 'ctx>),
    {
        let grid_id = make_label_id(&self.id, "__grid", &mut self.label_counts);
        let mut child_grid = Grid::new(grid_id.clone(), self.window, cols);
        f(&mut child_grid);
        child_grid.finish();
        self.children.push(grid_id);
    }
}
