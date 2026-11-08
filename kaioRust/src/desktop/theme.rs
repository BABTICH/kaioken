use serde::{Deserialize, Serialize};

/// Visual theme specification for the desktop application.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DesktopTheme {
    pub name: String,
    // Backgrounds & Surfaces
    pub background: String,
    pub surface: String,
    pub surface_hover: String,
    pub surface_active: String,
    pub border: String,
    pub border_focus: String,

    // Typography
    pub text_primary: String,
    pub text_secondary: String,
    pub text_muted: String,
    pub text_accent: String,

    // Status & Accents
    pub accent_blue: String,
    pub accent_green: String,
    pub accent_yellow: String,
    pub accent_red: String,
    pub accent_purple: String,

    // Code & Diff Viewer
    pub diff_add_bg: String,
    pub diff_add_fg: String,
    pub diff_del_bg: String,
    pub diff_del_fg: String,
    pub gutter_bg: String,
    pub gutter_fg: String,
}

impl Default for DesktopTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl DesktopTheme {
    /// Sleek Zed/GitHub-inspired high-contrast dark theme.
    pub fn dark() -> Self {
        Self {
            name: "Kaioken Dark".to_string(),
            background: "#0d1117".to_string(),
            surface: "#161b22".to_string(),
            surface_hover: "#21262d".to_string(),
            surface_active: "#30363d".to_string(),
            border: "#30363d".to_string(),
            border_focus: "#58a6ff".to_string(),

            text_primary: "#f0f6fc".to_string(),
            text_secondary: "#c9d1d9".to_string(),
            text_muted: "#8b949e".to_string(),
            text_accent: "#58a6ff".to_string(),

            accent_blue: "#58a6ff".to_string(),
            accent_green: "#3fb950".to_string(),
            accent_yellow: "#d29922".to_string(),
            accent_red: "#f85149".to_string(),
            accent_purple: "#bc8cff".to_string(),

            diff_add_bg: "#13231b".to_string(),
            diff_add_fg: "#3fb950".to_string(),
            diff_del_bg: "#2b1016".to_string(),
            diff_del_fg: "#f85149".to_string(),
            gutter_bg: "#161b22".to_string(),
            gutter_fg: "#6e7681".to_string(),
        }
    }

    /// Clean modern light theme option.
    pub fn light() -> Self {
        Self {
            name: "Kaioken Light".to_string(),
            background: "#ffffff".to_string(),
            surface: "#f6f8fa".to_string(),
            surface_hover: "#eaeef2".to_string(),
            surface_active: "#d0d7de".to_string(),
            border: "#d0d7de".to_string(),
            border_focus: "#0969da".to_string(),

            text_primary: "#1f2328".to_string(),
            text_secondary: "#24292f".to_string(),
            text_muted: "#656d76".to_string(),
            text_accent: "#0969da".to_string(),

            accent_blue: "#0969da".to_string(),
            accent_green: "#1a7f37".to_string(),
            accent_yellow: "#9a6700".to_string(),
            accent_red: "#cf222e".to_string(),
            accent_purple: "#8250df".to_string(),

            diff_add_bg: "#e6ffec".to_string(),
            diff_add_fg: "#1a7f37".to_string(),
            diff_del_bg: "#ffebe9".to_string(),
            diff_del_fg: "#cf222e".to_string(),
            gutter_bg: "#f6f8fa".to_string(),
            gutter_fg: "#8c959f".to_string(),
        }
    }
}
