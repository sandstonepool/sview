//! Color theme system for sview TUI
//!
//! Provides multiple pastel color themes for both light and dark modes.
//! Themes are easily switchable at runtime and persistable in config.

use ratatui::prelude::Color;
use serde::{Deserialize, Serialize};

/// Available color themes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Default dark theme (cool blues and greens)
    #[default]
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
    #[allow(dead_code)]
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

/// Color palette for a theme
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Palette {
    // Primary colors
    pub primary: Color,   // Main accent (cyan, teal, purple, etc.)
    pub secondary: Color, // Alternative accent (magenta, orange, etc.)
    pub tertiary: Color,  // Third accent (yellow, pink, etc.)

    // Status colors (health indicators)
    pub healthy: Color,  // Good status (green)
    pub warning: Color,  // Warning status (yellow/orange)
    pub critical: Color, // Critical status (red)

    // UI element colors
    pub border: Color,     // Border & divider lines
    pub text: Color,       // Primary text
    pub text_muted: Color, // Secondary/disabled text
    pub background: Color, // Background (mostly for light mode)

    // Special colors
    pub sparkline: Color, // Sparkline graphs
    pub gauge: Color,     // Gauge/progress bar
    pub gauge_bg: Color,  // Gauge background (for proper contrast)
}

impl Palette {
    /// Dark Default: Cool blues and greens (classic hacker aesthetic)
    /// High contrast: bright colors on dark background
    fn dark_default() -> Self {
        Palette {
            primary: Color::Rgb(139, 233, 253),    // Bright cyan
            secondary: Color::Rgb(189, 147, 249),  // Bright purple
            tertiary: Color::Rgb(255, 198, 109),   // Bright orange
            healthy: Color::Rgb(80, 250, 123),     // Bright green (improved)
            warning: Color::Rgb(255, 230, 100),    // Bright yellow (improved)
            critical: Color::Rgb(255, 85, 85),     // Bright red (improved)
            border: Color::Rgb(98, 114, 164),      // Blue-gray border
            text: Color::Rgb(248, 248, 242),       // Near-white text
            text_muted: Color::Rgb(150, 150, 170), // Lighter muted (improved contrast)
            background: Color::Black,
            sparkline: Color::Rgb(139, 233, 253),
            gauge: Color::Rgb(80, 250, 123),
            gauge_bg: Color::Rgb(40, 42, 54), // Dark but not black
        }
    }

    /// Dark Warm: Oranges, pinks, and warm tones
    fn dark_warm() -> Self {
        Palette {
            primary: Color::Rgb(255, 179, 155),    // Coral
            secondary: Color::Rgb(255, 121, 198),  // Bright pink (improved)
            tertiary: Color::Rgb(255, 210, 126),   // Peach
            healthy: Color::Rgb(152, 251, 152),    // Pale green (improved)
            warning: Color::Rgb(255, 218, 85),     // Brighter gold (improved)
            critical: Color::Rgb(255, 110, 100),   // Brighter salmon (improved)
            border: Color::Rgb(140, 100, 80),      // Warm brown
            text: Color::Rgb(255, 245, 238),       // Seashell white (warmer)
            text_muted: Color::Rgb(180, 160, 140), // Warmer gray (improved contrast)
            background: Color::Black,
            sparkline: Color::Rgb(255, 179, 155),
            gauge: Color::Rgb(255, 210, 126),
            gauge_bg: Color::Rgb(50, 40, 35), // Warm dark brown
        }
    }

    /// Dark Purple: Purple-dominant pastel theme
    fn dark_purple() -> Self {
        Palette {
            primary: Color::Rgb(209, 172, 255),   // Brighter purple (improved)
            secondary: Color::Rgb(255, 150, 200), // Brighter magenta (improved)
            tertiary: Color::Rgb(255, 183, 177),  // Pink
            healthy: Color::Rgb(120, 255, 214),   // Bright cyan-green (improved)
            warning: Color::Rgb(255, 220, 100),   // Brighter yellow (improved)
            critical: Color::Rgb(255, 120, 140),  // Brighter red-pink (improved)
            border: Color::Rgb(120, 100, 160),    // Purple-gray (improved)
            text: Color::Rgb(248, 248, 255),      // Ghost white
            text_muted: Color::Rgb(160, 140, 180), // Lighter purple-muted (improved)
            background: Color::Black,
            sparkline: Color::Rgb(209, 172, 255),
            gauge: Color::Rgb(255, 150, 200),
            gauge_bg: Color::Rgb(45, 35, 55), // Dark purple
        }
    }

    /// Dark Teal: Teal and cyan-dominant theme
    fn dark_teal() -> Self {
        Palette {
            primary: Color::Rgb(100, 255, 218),    // Brighter teal (improved)
            secondary: Color::Rgb(135, 206, 250),  // Light sky blue (improved)
            tertiary: Color::Rgb(189, 224, 254),   // Periwinkle
            healthy: Color::Rgb(144, 238, 144),    // Light green (improved)
            warning: Color::Rgb(255, 235, 100),    // Brighter lime (improved)
            critical: Color::Rgb(255, 130, 120),   // Brighter coral (improved)
            border: Color::Rgb(95, 158, 160),      // Cadet blue (improved)
            text: Color::Rgb(245, 255, 255),       // Azure white
            text_muted: Color::Rgb(150, 180, 180), // Lighter teal-muted (improved)
            background: Color::Black,
            sparkline: Color::Rgb(100, 255, 218),
            gauge: Color::Rgb(135, 206, 250),
            gauge_bg: Color::Rgb(35, 50, 50), // Dark teal
        }
    }

    /// Light Default: Deep saturated colors on light background
    /// High contrast: dark/saturated colors for readability
    fn light_default() -> Self {
        Palette {
            primary: Color::Rgb(30, 90, 180),    // Deep blue (improved)
            secondary: Color::Rgb(140, 50, 180), // Deep purple (improved)
            tertiary: Color::Rgb(180, 100, 20),  // Deep brown/orange (improved)
            healthy: Color::Rgb(30, 130, 50),    // Deep green (improved)
            warning: Color::Rgb(180, 130, 0),    // Deep gold (improved)
            critical: Color::Rgb(200, 40, 40),   // Deep red (improved)
            border: Color::Rgb(100, 100, 120),   // Darker gray-blue (improved)
            text: Color::Rgb(20, 20, 30),        // Near-black text (improved)
            text_muted: Color::Rgb(80, 80, 100), // Dark gray (improved contrast)
            background: Color::Rgb(250, 250, 255),
            sparkline: Color::Rgb(30, 90, 180),
            gauge: Color::Rgb(30, 130, 50),
            gauge_bg: Color::Rgb(220, 220, 230), // Light gray for gauge bg
        }
    }

    /// Light Warm: Deep warm colors on cream background
    fn light_warm() -> Self {
        Palette {
            primary: Color::Rgb(180, 80, 60),    // Deep coral (improved)
            secondary: Color::Rgb(170, 50, 100), // Deep rose (improved)
            tertiary: Color::Rgb(180, 110, 30),  // Deep orange (improved)
            healthy: Color::Rgb(40, 140, 60),    // Deep green (improved)
            warning: Color::Rgb(180, 130, 0),    // Deep amber (improved)
            critical: Color::Rgb(200, 50, 50),   // Deep red (improved)
            border: Color::Rgb(150, 120, 100),   // Darker warm beige (improved)
            text: Color::Rgb(40, 25, 15),        // Very dark warm (improved)
            text_muted: Color::Rgb(100, 80, 60), // Dark warm gray (improved contrast)
            background: Color::Rgb(255, 250, 245),
            sparkline: Color::Rgb(180, 80, 60),
            gauge: Color::Rgb(180, 110, 30),
            gauge_bg: Color::Rgb(235, 225, 215), // Warm light gray
        }
    }

    /// Light Cool: Deep cool colors on cool white background
    fn light_cool() -> Self {
        Palette {
            primary: Color::Rgb(0, 130, 110),    // Deep teal (improved)
            secondary: Color::Rgb(50, 100, 170), // Deep sky blue (improved)
            tertiary: Color::Rgb(120, 90, 160),  // Deep lavender (improved)
            healthy: Color::Rgb(30, 140, 70),    // Deep green (improved)
            warning: Color::Rgb(180, 140, 0),    // Deep gold (improved)
            critical: Color::Rgb(200, 60, 60),   // Deep red (improved)
            border: Color::Rgb(100, 130, 140),   // Darker cool gray (improved)
            text: Color::Rgb(20, 35, 45),        // Very dark cool (improved)
            text_muted: Color::Rgb(70, 95, 110), // Dark cool gray (improved contrast)
            background: Color::Rgb(245, 250, 252),
            sparkline: Color::Rgb(0, 130, 110),
            gauge: Color::Rgb(50, 100, 170),
            gauge_bg: Color::Rgb(220, 230, 235), // Cool light gray
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
