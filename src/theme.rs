use ratatui::style::Color;
use crate::app::Theme;

pub struct ThemeColors {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub cpu: Color,
    pub memory: Color,
    pub network: Color,
    pub disk: Color,
    pub warning: Color,
    pub danger: Color,
    pub success: Color,
    pub text: Color,
    pub text_dim: Color,
    pub border: Color,
    pub highlight_bg: Color,
    pub tab_active: Color,
}

impl ThemeColors {
    pub fn from_theme(theme: Theme) -> Self {
        match theme {
            Theme::Default => Self {
                primary: Color::Cyan,
                secondary: Color::Magenta,
                accent: Color::Yellow,
                cpu: Color::Cyan,
                memory: Color::Magenta,
                network: Color::Green,
                disk: Color::Yellow,
                warning: Color::Yellow,
                danger: Color::Red,
                success: Color::Green,
                text: Color::White,
                text_dim: Color::Gray,
                border: Color::DarkGray,
                highlight_bg: Color::DarkGray,
                tab_active: Color::Cyan,
            },
            Theme::Ocean => Self {
                primary: Color::Rgb(100, 180, 255),
                secondary: Color::Rgb(130, 160, 255),
                accent: Color::Rgb(0, 220, 200),
                cpu: Color::Rgb(100, 180, 255),
                memory: Color::Rgb(130, 160, 255),
                network: Color::Rgb(0, 220, 200),
                disk: Color::Rgb(180, 200, 255),
                warning: Color::Rgb(255, 200, 100),
                danger: Color::Rgb(255, 100, 100),
                success: Color::Rgb(100, 255, 180),
                text: Color::Rgb(220, 230, 255),
                text_dim: Color::Rgb(120, 140, 170),
                border: Color::Rgb(60, 80, 120),
                highlight_bg: Color::Rgb(30, 50, 80),
                tab_active: Color::Rgb(100, 180, 255),
            },
            Theme::Forest => Self {
                primary: Color::Rgb(100, 200, 100),
                secondary: Color::Rgb(180, 220, 100),
                accent: Color::Rgb(255, 200, 80),
                cpu: Color::Rgb(100, 200, 100),
                memory: Color::Rgb(180, 220, 100),
                network: Color::Rgb(80, 180, 160),
                disk: Color::Rgb(200, 180, 100),
                warning: Color::Rgb(255, 200, 80),
                danger: Color::Rgb(255, 100, 80),
                success: Color::Rgb(100, 220, 120),
                text: Color::Rgb(220, 240, 220),
                text_dim: Color::Rgb(120, 160, 120),
                border: Color::Rgb(60, 100, 60),
                highlight_bg: Color::Rgb(30, 60, 30),
                tab_active: Color::Rgb(100, 200, 100),
            },
            Theme::Sunset => Self {
                primary: Color::Rgb(255, 150, 80),
                secondary: Color::Rgb(255, 100, 130),
                accent: Color::Rgb(255, 200, 100),
                cpu: Color::Rgb(255, 150, 80),
                memory: Color::Rgb(255, 100, 130),
                network: Color::Rgb(255, 180, 60),
                disk: Color::Rgb(200, 150, 255),
                warning: Color::Rgb(255, 220, 100),
                danger: Color::Rgb(255, 80, 80),
                success: Color::Rgb(100, 220, 150),
                text: Color::Rgb(255, 240, 230),
                text_dim: Color::Rgb(180, 140, 120),
                border: Color::Rgb(120, 80, 60),
                highlight_bg: Color::Rgb(80, 40, 30),
                tab_active: Color::Rgb(255, 150, 80),
            },
        }
    }

    pub fn cpu_usage_color(&self, usage: f64) -> Color {
        if usage > 80.0 {
            self.danger
        } else if usage > 50.0 {
            self.warning
        } else {
            self.success
        }
    }

    pub fn disk_usage_color(&self, pct: f64) -> Color {
        if pct > 90.0 {
            self.danger
        } else if pct > 70.0 {
            self.warning
        } else {
            self.success
        }
    }
}
