use std::str::FromStr;

use anyhow::Context;
use ratatui::style::{Color, ParseColorError, Style};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct StyleConfig {
    pub fg: Option<String>,
    pub bg: Option<String>,
    pub bold: bool,
    pub italic: bool,
}

impl StyleConfig {
    pub fn format(&self) -> anyhow::Result<Style> {
        let style: Style = self.clone().try_into().with_context(|| {
            format!(
                "invalid style in config (fg: {:?}, bg: {:?}, bold: {:?}, italic: {:?})",
                self.fg, self.bg, self.bold, self.italic
            )
        })?;
        Ok(style)
    }
}

impl TryFrom<StyleConfig> for Style {
    type Error = ParseColorError;

    fn try_from(config: StyleConfig) -> Result<Self, Self::Error> {
        let mut style = Style::new();
        if let Some(fg_str) = config.fg {
            let color = Color::from_str(&fg_str)?;
            style = style.fg(color);
        }

        if let Some(bg_str) = config.bg {
            let color = Color::from_str(&bg_str)?;
            style = style.bg(color);
        }

        if config.bold {
            style = style.bold();
        }

        if config.italic {
            style = style.italic();
        }

        Ok(style)
    }
}
