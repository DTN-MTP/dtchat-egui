use egui::Color32;

pub trait Color32FromStr {
    fn from_str(color: &str) -> Self;
}

impl Color32FromStr for Color32 {
    fn from_str(color: &str) -> Self {
        match color {
            "TRANSPARENT" => Color32::TRANSPARENT,
            "BLACK" => Color32::BLACK,
            "DARK_GRAY" => Color32::DARK_GRAY,
            "GRAY" => Color32::GRAY,
            "LIGHT_GRAY" => Color32::LIGHT_GRAY,
            "WHITE" => Color32::WHITE,
            "BROWN" => Color32::BROWN,
            "DARK_RED" => Color32::DARK_RED,
            "RED" => Color32::RED,
            "LIGHT_RED" => Color32::LIGHT_RED,
            "CYAN" => Color32::CYAN,
            "MAGENTA" => Color32::MAGENTA,
            "YELLOW" => Color32::YELLOW,
            "ORANGE" => Color32::ORANGE,
            "LIGHT_YELLOW" => Color32::LIGHT_YELLOW,
            "KHAKI" => Color32::KHAKI,
            "DARK_GREEN" => Color32::DARK_GREEN,
            "GREEN" => Color32::GREEN,
            "LIGHT_GREEN" => Color32::LIGHT_GREEN,
            "DARK_BLUE" => Color32::DARK_BLUE,
            "BLUE" => Color32::BLUE,
            "LIGHT_BLUE" => Color32::LIGHT_BLUE,
            "PURPLE" => Color32::PURPLE,
            "GOLD" => Color32::GOLD,
            "DEBUG_COLOR" => Color32::DEBUG_COLOR,
            _ => Color32::DEBUG_COLOR,
        }
    }
}
