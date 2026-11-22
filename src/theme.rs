// FlClash-style palette (iOS system colors + #007AFF accent), applied on top
// of gpui-component's theme so built-in widgets follow the same tokens.
use gpui::App;
use gpui_component::{Theme, ThemeMode};

const LIGHT_JSON: &str = r##"{
  "tokens": {
    "colors": {
      "background": "#F4F3F9",
      "foreground": "#1C1C1E",
      "surface": "#FFFFFF",
      "surface_foreground": "#1C1C1E",
      "primary": "#007AFF",
      "primary_foreground": "#FFFFFF",
      "secondary": "#F2F2F7",
      "secondary_foreground": "#1C1C1E",
      "muted": "#EFEFF4",
      "muted_foreground": "#8A8A8F",
      "accent": "#EAF2FF",
      "accent_foreground": "#007AFF",
      "destructive": "#FF3B30",
      "destructive_foreground": "#FFFFFF",
      "border": "#E4E4EA",
      "input": "#E4E4EA",
      "ring": "#007AFF"
    },
    "radius": { "none": 0, "sm": 8, "md": 10, "lg": 16, "xl": 20, "full": 999 },
    "typography": { "sans": "Segoe UI" }
  }
}"##;

const DARK_JSON: &str = r##"{
  "tokens": {
    "colors": {
      "background": "#121214",
      "foreground": "#F2F2F7",
      "surface": "#1C1C1E",
      "surface_foreground": "#F2F2F7",
      "primary": "#0A84FF",
      "primary_foreground": "#FFFFFF",
      "secondary": "#2C2C2E",
      "secondary_foreground": "#F2F2F7",
      "muted": "#26262A",
      "muted_foreground": "#98989F",
      "accent": "#1E2C3F",
      "accent_foreground": "#64A8FF",
      "destructive": "#FF453A",
      "destructive_foreground": "#FFFFFF",
      "border": "#333338",
      "input": "#3A3A40",
      "ring": "#0A84FF"
    },
    "radius": { "none": 0, "sm": 8, "md": 10, "lg": 16, "xl": 20, "full": 999 },
    "typography": { "sans": "Segoe UI" }
  }
}"##;

pub fn apply(cx: &mut App, mode: ThemeMode) {
    Theme::change(mode, None, cx);
    apply_palette(cx, mode);
}

pub fn apply_palette(cx: &mut App, mode: ThemeMode) {
    let json = if mode.is_dark() { DARK_JSON } else { LIGHT_JSON };
    Theme::update(cx, |theme| {
        let _ = theme.apply_semantic_config_str(json);
        // Mirror FlClash: primary buttons/controls use the accent blue.
        let p = theme.primary;
        let pf = theme.primary_foreground;
        theme.button_primary = p;
        theme.button_primary_foreground = pf;
        theme.button_primary_hover = p.opacity(0.9);
        theme.button_primary_active = p.opacity(0.8);
        theme.primary_hover = p.opacity(0.9);
        theme.primary_active = p.opacity(0.8);
    });
}

/// FlClash surface accents that the semantic tokens do not cover.
pub struct Surfaces {
    pub sidebar: u32,
    pub sidebar_pill: u32,
    pub sidebar_text: u32,
    pub card_border: u32,
    pub hover: u32,
    pub chevron: u32,
}

pub fn surfaces(dark: bool) -> Surfaces {
    if dark {
        Surfaces {
            sidebar: 0x17171A,
            sidebar_pill: 0x26262A,
            sidebar_text: 0xD8D8DE,
            card_border: 0x2A2A2E,
            hover: 0x242428,
            chevron: 0x8A8A90,
        }
    } else {
        Surfaces {
            sidebar: 0xECECF2,
            sidebar_pill: 0xE0E0E8,
            sidebar_text: 0x3C3C43,
            card_border: 0xEDEDF2,
            hover: 0xF0F0F5,
            chevron: 0xB0B0B8,
        }
    }
}
