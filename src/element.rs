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
    /// Progress value (0.0 to 1.0)
    Progress(f64),
    /// Stat card value with optional subvalue
    StatValue { value: String, subvalue: Option<String> },
    /// Status indicator state
    StatusValue {
        active: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        active_text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        inactive_text: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        active_color: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        inactive_color: Option<String>,
    },
    /// Mini chart data
    ChartValue {
        values: Vec<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        current: Option<f32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        unit: Option<String>,
    },
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
    /// Section header for grouping
    Section,
    /// Progress bar with percentage
    ProgressBar,
    /// Stat card display
    Stat,
    /// Status indicator with colored dot
    Status,
    /// Mini sparkline chart
    MiniChart,
    /// Grid layout container
    Grid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ElementMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    /// Accent color for the element (coral, teal, blue, green, purple, orange, yellow, red)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent: Option<String>,
    /// Subtitle or secondary text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    /// Number of columns for grid layout
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cols: Option<usize>,
    /// Child element IDs for grid layout
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<String>>,
}

impl Default for ElementMeta {
    fn default() -> Self {
        Self {
            min: None,
            max: None,
            step: None,
            accent: None,
            subtitle: None,
            cols: None,
            children: None,
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

/// Accent colors available for UI elements
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccentColor {
    Coral,
    Teal,
    Blue,
    Green,
    Purple,
    Orange,
    Yellow,
    Red,
}

impl AccentColor {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccentColor::Coral => "coral",
            AccentColor::Teal => "teal",
            AccentColor::Blue => "blue",
            AccentColor::Green => "green",
            AccentColor::Purple => "purple",
            AccentColor::Orange => "orange",
            AccentColor::Yellow => "yellow",
            AccentColor::Red => "red",
        }
    }
}
