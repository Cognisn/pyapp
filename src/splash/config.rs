pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn from_hex(hex: &str) -> Color {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        Color { r, g, b }
    }

    pub fn to_egui(&self) -> [u8; 3] {
        [self.r, self.g, self.b]
    }
}

pub struct SplashConfig {
    pub bg_color: Color,
    pub text_color: Color,
    pub progress_color: Color,
    pub window_title: String,
    pub window_width: u32,
    pub window_height: u32,
    pub project_name: String,
    pub project_version: String,
}

impl SplashConfig {
    pub fn from_env() -> SplashConfig {
        SplashConfig {
            bg_color: Color::from_hex(env!("PYAPP_SPLASH_BG_COLOR")),
            text_color: Color::from_hex(env!("PYAPP_SPLASH_TEXT_COLOR")),
            progress_color: Color::from_hex(env!("PYAPP_SPLASH_PROGRESS_COLOR")),
            window_title: env!("PYAPP_SPLASH_WINDOW_TITLE").to_string(),
            window_width: env!("PYAPP_SPLASH_WINDOW_WIDTH")
                .parse()
                .unwrap_or(480),
            window_height: env!("PYAPP_SPLASH_WINDOW_HEIGHT")
                .parse()
                .unwrap_or(360),
            project_name: env!("PYAPP_PROJECT_NAME").to_string(),
            project_version: env!("PYAPP_PROJECT_VERSION").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hex_color_with_hash() {
        let color = Color::from_hex("#1a1a2e");
        assert_eq!(color.r, 0x1a);
        assert_eq!(color.g, 0x1a);
        assert_eq!(color.b, 0x2e);
    }

    #[test]
    fn test_parse_hex_color_without_hash() {
        let color = Color::from_hex("ffffff");
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 255);
    }

    #[test]
    fn test_parse_hex_color_black() {
        let color = Color::from_hex("#000000");
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_to_egui() {
        let color = Color::from_hex("#4a90d9");
        assert_eq!(color.to_egui(), [0x4a, 0x90, 0xd9]);
    }
}
