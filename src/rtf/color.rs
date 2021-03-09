#[derive(Clone, Debug, PartialEq)]
pub struct Color {
    pub b: u8,
    pub g: u8,
    pub r: u8,
}

impl Default for Color {
    fn default() -> Color {
        Color { b: 0, g: 0, r: 0 }
    }
}

impl Into<String> for Color {
    fn into(self) -> String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}
impl Into<String> for &Color {
    fn into(self) -> String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}
