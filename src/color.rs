//! Colored, optionally emoji-decorated console output, with custom
//! hex (`#RRGGBB`) colors for each message category.

use colored::{Color, Colorize};
use is_terminal::IsTerminal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl std::str::FromStr for ColorMode {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "auto" => Ok(ColorMode::Auto),
            "always" => Ok(ColorMode::Always),
            "never" => Ok(ColorMode::Never),
            other => Err(format!(
                "invalid value '{other}' for --color, expected auto|always|never"
            )),
        }
    }
}

/// Decide whether colored output should be used, honoring `NO_COLOR` and
/// whether stdout is an interactive terminal.
pub fn should_use_color(mode: ColorMode) -> bool {
    match mode {
        ColorMode::Always => true,
        ColorMode::Never => false,
        ColorMode::Auto => {
            std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
        }
    }
}

/// Parse a `#RRGGBB` or `RRGGBB` hex string into an RGB triple.
pub fn parse_hex(s: &str) -> Result<(u8, u8, u8), String> {
    let trimmed = s.trim().trim_start_matches('#');
    if trimmed.len() != 6 || !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "invalid hex color '{s}', expected a format like #00FFFF"
        ));
    }
    let r = u8::from_str_radix(&trimmed[0..2], 16).unwrap();
    let g = u8::from_str_radix(&trimmed[2..4], 16).unwrap();
    let b = u8::from_str_radix(&trimmed[4..6], 16).unwrap();
    Ok((r, g, b))
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub success: (u8, u8, u8),
    pub error: (u8, u8, u8),
    pub warn: (u8, u8, u8),
    pub info: (u8, u8, u8),
    pub enabled: bool,
    pub emoji: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            success: (0x00, 0xC8, 0x53), // green
            error: (0xFF, 0x3B, 0x30),   // red
            warn: (0xFF, 0xA5, 0x00),    // amber
            info: (0x00, 0xAF, 0xFF),    // cyan-blue
            enabled: true,
            emoji: true,
        }
    }
}

impl Theme {
    fn paint(&self, text: &str, rgb: (u8, u8, u8)) -> String {
        if self.enabled {
            text.color(Color::TrueColor {
                r: rgb.0,
                g: rgb.1,
                b: rgb.2,
            })
            .to_string()
        } else {
            text.to_string()
        }
    }

    pub fn success(&self, text: &str) -> String {
        let icon = if self.emoji { "✅ " } else { "" };
        format!("{icon}{}", self.paint(text, self.success))
    }

    pub fn error(&self, text: &str) -> String {
        let icon = if self.emoji { "❌ " } else { "" };
        format!("{icon}{}", self.paint(text, self.error))
    }

    pub fn warn(&self, text: &str) -> String {
        let icon = if self.emoji { "⚠️  " } else { "" };
        format!("{icon}{}", self.paint(text, self.warn))
    }

    pub fn info(&self, text: &str) -> String {
        let icon = if self.emoji { "📁 " } else { "" };
        format!("{icon}{}", self.paint(text, self.info))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_with_hash() {
        assert_eq!(parse_hex("#00FFFF").unwrap(), (0, 255, 255));
    }

    #[test]
    fn parses_hex_without_hash() {
        assert_eq!(parse_hex("00ffff").unwrap(), (0, 255, 255));
    }

    #[test]
    fn parses_mixed_case() {
        assert_eq!(parse_hex("#AaBbCc").unwrap(), (0xAA, 0xBB, 0xCC));
    }

    #[test]
    fn rejects_invalid_hex() {
        assert!(parse_hex("#ZZZZZZ").is_err());
        assert!(parse_hex("#FFF").is_err());
        assert!(parse_hex("not-a-color").is_err());
    }

    #[test]
    fn color_mode_parses_known_values() {
        use std::str::FromStr;
        assert_eq!(ColorMode::from_str("auto").unwrap(), ColorMode::Auto);
        assert_eq!(ColorMode::from_str("ALWAYS").unwrap(), ColorMode::Always);
        assert_eq!(ColorMode::from_str("never").unwrap(), ColorMode::Never);
        assert!(ColorMode::from_str("rainbow").is_err());
    }
}
