use std::ops::RangeInclusive;

use crate::context::Context;
use crate::element::{ElementDecl, ElementKind, ElementMeta, Value};

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
        // Use a unique-ish id based on current element count
        let id = format!("{}::__sep_{}", self.name, self.ctx.current_frame_len());

        self.ctx.declare(ElementDecl {
            id,
            kind: ElementKind::Separator,
            label: String::new(),
            value: Value::Bool(false), // unused
            meta: ElementMeta::default(),
            window: self.name.clone(),
        });
    }
}
