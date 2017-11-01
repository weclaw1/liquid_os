#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Color {
    Black      = 0x0,
    Blue       = 0x1,
    Green      = 0x2,
    Cyan       = 0x3,
    Red        = 0x4,
    Magenta    = 0x5,
    Brown      = 0x6,
    LightGray  = 0x7,
    DarkGray   = 0x8,
    LightBlue  = 0x9,
    LightGreen = 0xA,
    LightCyan  = 0xB,
    LightRed   = 0xC,
    Pink       = 0xD,
    Yellow     = 0xE,
    White      = 0xF,
}

#[derive(Debug, Clone, Copy)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[cfg(test)]
mod tests {
    use color::Color;
    use color;

    #[test]
    fn colorcode() {
        assert_eq!(color::colorcode(Color::Blue, Color::BrightMagenta), 0xD1);
        assert_eq!(color::colorcode(Color::Yellow, Color::Red), 0x4E);
        assert_eq!(color::colorcode(Color::DarkGray, Color::White), 0xF8);
    }

}