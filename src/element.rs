use serde::{Deserialize, Serialize};

pub type ElementId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum Value {
    Float(f64),
    Bool(bool),
    Color3([f32; 3]),
    Color4([f32; 4]),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Int(i64),
    String(String),
    Enum {
        selected: usize,
        options: Vec<String>,
    },
    /// Transient: true for one frame when clicked
    Button(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum ElementKind {
    Slider,
    Checkbox,
    ColorPicker3,
    ColorPicker4,
    TextInput,
    Dropdown,
    Button,
    Label,
    Separator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElementMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
}

impl Default for ElementMeta {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            step: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementDecl {
    pub id: ElementId,
    pub kind: ElementKind,
    pub label: String,
    pub value: Value,
    pub meta: ElementMeta,
    pub window: String,
}
