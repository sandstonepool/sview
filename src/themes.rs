//! Color theme system for sview TUI
//!
//! Provides multiple pastel color themes for both light and dark modes.
//! Themes are easily switchable at runtime and persistable in config.

use ratatui::prelude::Color;
use serde::{Deserialize, Serialize};

/// Available color themes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Default dark theme (cool blues and greens)
    DarkDefault,
    /// Warm dark theme (oranges and pinks)
    DarkWarm,
    /// Purple-focused dark theme
    DarkPurple,
    /// Teal-focused dark theme
    DarkTeal,
    /// Light theme (soft pastels on light background)
    LightDefault,
    /// Warm light theme (peachy pastels)
    LightWarm,
    /// Cool light theme (minty pastels)
    LightCool,
}

impl Theme {
    /// Get all available themes
    pub fn all() -> &'static [Theme] {
        &[
            Theme::DarkDefault,
            Theme::DarkWarm,
            Theme::DarkPurple,
            Theme::DarkTeal,
            Theme::LightDefault,
            Theme::LightWarm,
            Theme::LightCool,
        ]
    }

    /// Get next theme in rotation
    pub fn next(&self) -> Theme {
        match self {
            Theme::DarkDefault => Theme::DarkWarm,
            Theme::DarkWarm => Theme::DarkPurple,
            Theme::DarkPurple => Theme::DarkTeal,
            Theme::DarkTeal => Theme::LightDefault,
            Theme::LightDefault => Theme::LightWarm,
            Theme::LightWarm => Theme::LightCool,
            Theme::LightCool => Theme::DarkDefault,
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &str {
        match self {
            Theme::DarkDefault => "Dark Default",
            Theme::DarkWarm => "Dark Warm",
            Theme::DarkPurple => "Dark Purple",
            Theme::DarkTeal => "Dark Teal",
            Theme::LightDefault => "Light Default",
            Theme::LightWarm => "Light Warm",
            Theme::LightCool => "Light Cool",
        }
    }

    /// Get color palette for this theme
    pub fn palette(&self) -> Palette {
        match self {
            Theme::DarkDefault => Palette::dark_default(),
            Theme::DarkWarm => Palette::dark_warm(),
            Theme::DarkPurple => Palette::dark_purple(),
            Theme::DarkTeal => Palette::dark_teal(),
            Theme::LightDefault => Palette::light_default(),
            Theme::LightWarm => Palette::light_warm(),
            Theme::LightCool => Palette::light_cool(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::DarkDefault
    }
}

/// Color palette for a theme
#[derive(Debug, Clone)]
pub struct Palette {
    // Primary colors
    pub primary: Color,      // Main accent (cyan, teal, purple, etc.)
    pub secondary: Color,    // Alternative accent (magenta, orange, etc.)
    pub tertiary: Color,     // Third accent (yellow, pink, etc.)

    // Status colors (health indicators)
    pub healthy: Color,      // Good status (green)
    pub warning: Color,      // Warning status (yellow/orange)
    pub critical: Color,     // Critical status (red)

    // UI element colors
    pub border: Color,       // Border & divider lines
    pub text: Color,         // Primary text
    pub text_muted: Color,   // Secondary/disabled text
    pub background: Color,   // Background (mostly for light mode)

    // Special colors
    pub sparkline: Color,    // Sparkline graphs
    pub gauge: Color,        // Gauge/progress bar
}

impl Palette {
    /// Dark Default: Cool blues and greens (classic hacker aesthetic)
    fn dark_default() -> Self {
        Palette {
            primary: Color::Rgb(139, 233, 253),      // Pastel cyan
            secondary: Color::Rgb(189, 147, 249),    // Pastel purple
            tertiary: Color::Rgb(255, 198, 109),     // Pastel orange
            healthy: Color::Rgb(166, 227, 161),      // Pastel green
            warning: Color::Rgb(249, 226, 175),      // Pastel yellow
            critical: Color::Rgb(255, 146, 146),     // Pastel red
            border: Color::Rgb(98, 112, 147),        // Muted blue-gray
            text: Color::Rgb(229, 229, 229),         // Light gray
            text_muted: Color::Rgb(128, 128, 128),   // Medium gray
            background: Color::Black,
            sparkline: Color::Rgb(139, 233, 253),    // Match primary
            gauge: Color::Rgb(166, 227, 161),        // Match healthy
        }
    }

    /// Dark Warm: Oranges, pinks, and warm tones
    fn dark_warm() -> Self {
        Palette {
            primary: Color::Rgb(255, 179, 155),      // Pastel coral
            secondary: Color::Rgb(255, 165, 200),    // Pastel rose
            tertiary: Color::Rgb(255, 210, 126),     // Pastel peach
            healthy: Color::Rgb(200, 238, 181),      // Pastel mint
            warning: Color::Rgb(255, 218, 138),      // Pastel gold
            critical: Color::Rgb(255, 150, 140),     // Pastel salmon
            border: Color::Rgb(140, 100, 80),        // Warm brown
            text: Color::Rgb(245, 235, 225),         // Warm white
            text_muted: Color::Rgb(130, 110, 90),    // Warm gray
            background: Color::Black,
            sparkline: Color::Rgb(255, 179, 155),    // Match primary
            gauge: Color::Rgb(255, 210, 126),        // Warm accent
        }
    }

    /// Dark Purple: Purple-dominant pastel theme
    fn dark_purple() -> Self {
        Palette {
            primary: Color::Rgb(209, 172, 221),      // Pastel purple
            secondary: Color::Rgb(233, 168, 201),    // Pastel magenta
            tertiary: Color::Rgb(255, 183, 177),     // Pastel pink
            healthy: Color::Rgb(178, 223, 219),      // Pastel cyan
            warning: Color::Rgb(255, 214, 110),      // Pastel yellow
            critical: Color::Rgb(255, 157, 167),     // Pastel red-pink
            border: Color::Rgb(100, 80, 130),        // Purple-gray
            text: Color::Rgb(240, 240, 245),         // Cool white
            text_muted: Color::Rgb(120, 100, 140),   // Purple-muted
            background: Color::Black,
            sparkline: Color::Rgb(209, 172, 221),    // Match primary
            gauge: Color::Rgb(233, 168, 201),        // Match secondary
        }
    }

    /// Dark Teal: Teal and cyan-dominant theme
    fn dark_teal() -> Self {
        Palette {
            primary: Color::Rgb(155, 235, 215),      // Pastel teal
            secondary: Color::Rgb(159, 222, 242),    // Pastel sky blue
            tertiary: Color::Rgb(189, 224, 254),     // Pastel periwinkle
            healthy: Color::Rgb(182, 244, 204),      // Pastel mint
            warning: Color::Rgb(255, 229, 121),      // Pastel lime
            critical: Color::Rgb(255, 162, 155),     // Pastel coral
            border: Color::Rgb(85, 130, 130),        // Teal-gray
            text: Color::Rgb(235, 245, 245),         // Cool white
            text_muted: Color::Rgb(110, 130, 130),   // Teal-muted
            background: Color::Black,
            sparkline: Color::Rgb(155, 235, 215),    // Match primary
            gauge: Color::Rgb(159, 222, 242),        // Sky blue
        }
    }

    /// Light Default: Soft pastels on light background
    fn light_default() -> Self {
        Palette {
            primary: Color::Rgb(100, 160, 200),      // Pastel blue
            secondary: Color::Rgb(180, 100, 200),    // Pastel purple
            tertiary: Color::Rgb(220, 150, 80),      // Pastel brown
            healthy: Color::Rgb(100, 180, 120),      // Pastel green
            warning: Color::Rgb(220, 180, 40),       // Pastel gold
            critical: Color::Rgb(220, 100, 100),     // Pastel red
            border: Color::Rgb(160, 160, 180),       // Light gray-blue
            text: Color::Rgb(40, 40, 50),            // Dark text
            text_muted: Color::Rgb(120, 120, 140),   // Medium gray
            background: Color::Rgb(250, 250, 255),   // Very light blue
            sparkline: Color::Rgb(100, 160, 200),    // Match primary
            gauge: Color::Rgb(100, 180, 120),        // Match healthy
        }
    }

    /// Light Warm: Peachy and warm pastels
    fn light_warm() -> Self {
        Palette {
            primary: Color::Rgb(220, 140, 120),      // Pastel coral
            secondary: Color::Rgb(220, 120, 160),    // Pastel rose
            tertiary: Color::Rgb(240, 180, 110),     // Pastel peach
            healthy: Color::Rgb(140, 200, 140),      // Pastel green
            warning: Color::Rgb(240, 200, 100),      // Pastel yellow
            critical: Color::Rgb(240, 120, 100),     // Pastel salmon
            border: Color::Rgb(200, 150, 130),       // Warm beige
            text: Color::Rgb(50, 30, 20),            // Warm dark text
            text_muted: Color::Rgb(140, 100, 80),    // Warm gray
            background: Color::Rgb(255, 250, 245),   // Warm white
            sparkline: Color::Rgb(220, 140, 120),    // Match primary
            gauge: Color::Rgb(240, 180, 110),        // Warm accent
        }
    }

    /// Light Cool: Minty and cool pastels
    fn light_cool() -> Self {
        Palette {
            primary: Color::Rgb(120, 200, 180),      // Pastel mint
            secondary: Color::Rgb(140, 180, 220),    // Pastel sky
            tertiary: Color::Rgb(200, 180, 220),     // Pastel lavender
            healthy: Color::Rgb(140, 210, 160),      // Pastel green
            warning: Color::Rgb(230, 210, 120),      // Pastel yellow
            critical: Color::Rgb(240, 140, 130),     // Pastel red
            border: Color::Rgb(140, 170, 170),       // Cool gray
            text: Color::Rgb(30, 50, 60),            // Cool dark text
            text_muted: Color::Rgb(100, 130, 140),   // Cool gray
            background: Color::Rgb(245, 250, 252),   // Cool white
            sparkline: Color::Rgb(120, 200, 180),    // Match primary
            gauge: Color::Rgb(140, 180, 220),        // Sky blue
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_cycle() {
        let mut theme = Theme::DarkDefault;
        for _ in 0..7 {
            theme = theme.next();
        }
        assert_eq!(theme, Theme::DarkDefault);
    }

    #[test]
    fn test_all_themes_have_names() {
        for theme in Theme::all() {
            assert!(!theme.display_name().is_empty());
        }
    }

    #[test]
    fn test_palette_colors_are_distinct() {
        let palette = Theme::DarkDefault.palette();
        // Ensure primary and secondary are different
        assert_ne!(palette.primary, palette.secondary);
        assert_ne!(palette.healthy, palette.critical);
    }
}
