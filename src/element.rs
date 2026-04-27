use std::sync::Arc;

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
    StatValue {
        value: String,
        subvalue: Option<String>,
    },
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
    /// Grid container data
    GridValue {
        cols: usize,
        children: Vec<String>,
    },
    /// Plot data for larger charts
    PlotValue {
        series: Vec<PlotSeries>,
        #[serde(skip_serializing_if = "Option::is_none")]
        x_label: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        y_label: Option<String>,
    },
    /// Null value for container elements
    Null,
    /// Image data as a base64 data URI (e.g. "data:image/png;base64,…")
    ImageValue {
        data: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        width: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        height: Option<u32>,
    },
}

/// A data series for plotting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlotSeries {
    pub name: String,
    pub values: Vec<f32>,
    pub color: String,
    /// Whether this series should use relative autoscaling
    /// When true, the series is scaled independently
    /// When false, the series uses the plot's shared scale
    #[serde(default = "default_autoscale")]
    pub autoscale: bool,
}

fn default_autoscale() -> bool {
    true
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
    /// Larger plot/chart for data visualization
    Plot,
    /// Compact key-value display
    KeyValue,
    /// Compact button for dense UIs
    ButtonCompact,
    /// Horizontal layout container
    Horizontal,
    /// Inline button without label column (for horizontal layouts)
    ButtonInline,
    /// Inline text input without label column (for horizontal layouts)
    TextInputInline,
    /// Inline label for horizontal layouts (no wrapping)
    LabelInline,
    /// Image display widget
    Image,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ElementMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<f64>,
    /// Accent color for the element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent: Option<AccentColor>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ElementDecl {
    pub id: ElementId,
    pub kind: ElementKind,
    pub label: String,
    pub value: Value,
    pub meta: ElementMeta,
    pub window: Arc<str>,
}

/// Accent colors available for UI elements
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
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
