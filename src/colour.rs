#[derive(Debug, PartialEq)]
pub struct Colour {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Colour {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Colour { r, g, b, a: 0 }
    }

    pub fn from_u32(rgb: u32) -> Self {
        let r = ((rgb & 0xff0000) >> 16) as u8;
        let g = ((rgb & 0xff00) >> 8) as u8;
        let b = (rgb & 0xff) as u8;

        Colour { r, g, b, a: 0 }
    }

    pub fn as_0rgb(&self) -> u32 {
        ((self.r as u32) << 16) + ((self.g as u32) << 8) + (self.b as u32)
    }

    pub fn scale(&self, factor: f32) -> Self {
        let r = ((self.r as f32) * factor) as u8;
        let g = ((self.g as f32) * factor) as u8;
        let b = ((self.b as f32) * factor) as u8;

        Colour { r, g, b, a: 0 }
    }
}
