use clap::Parser;

#[derive(Clone)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl Into<[u8; 4]> for &Color {
    /// Translate this to byte reprensation,
    /// in ARGB8888 format.
    fn into(self) -> [u8; 4] {
        [self.a, self.r, self.g, self.b]
    }
}

impl core::str::FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.get(0..1).is_some_and(|v| v == "#") {
            return Err(Into::<String>::into("Invalid string"));
        }
        Ok(match s.len() {
            4 => {
                let number = u32::from_str_radix(
                    s.get(1..).ok_or(Into::<String>::into("Invalid string"))?,
                    16,
                )
                .map_err(|_| Into::<String>::into("Invalid string"))?;
                Color::new(
                    ((number & 0xf00) >> 8)
                        .try_into()
                        .map_err(|_| Into::<String>::into("Invalid string"))?,
                    ((number & 0x0f0) >> 4)
                        .try_into()
                        .map_err(|_| Into::<String>::into("Invalid string"))?,
                    (number & 0x00f)
                        .try_into()
                        .map_err(|_| Into::<String>::into("Invalid string"))?,
                    0xff,
                )
            }
            5 => {
                let number = u32::from_str_radix(
                    s.get(1..).ok_or(Into::<String>::into("Invalid string"))?,
                    16,
                )
                .map_err(|_| Into::<String>::into("Invalid string"))?;
                Color::new(
                    ((number & 0x0f00) >> 8)
                        .try_into()
                        .map_err(|_| Into::<String>::into("Invalid string"))?,
                    ((number & 0x00f0) >> 4)
                        .try_into()
                        .map_err(|_| Into::<String>::into("Invalid string"))?,
                    (number & 0x000f)
                        .try_into()
                        .map_err(|_| Into::<String>::into("Invalid string"))?,
                    ((number & 0xf000) >> 12)
                        .try_into()
                        .map_err(|_| Into::<String>::into("Invalid string"))?,
                )
            }
            7 => {
                let number = u32::from_str_radix(
                    s.get(1..).ok_or(Into::<String>::into("Invalid string"))?,
                    16,
                )
                .map_err(|_| Into::<String>::into("Invalid string"))?;
                Color::new(
                    ((number & 0xff0000) >> 8)
                        .try_into()
                        .map_err(|_| Into::<String>::into("Invalid string"))?,
                    ((number & 0x00ff00) >> 4)
                        .try_into()
                        .map_err(|_| Into::<String>::into("Invalid string"))?,
                    (number & 0x0000ff)
                        .try_into()
                        .map_err(|_| Into::<String>::into("Invalid string"))?,
                    0xff,
                )
            }
            9 => {
                let number = u32::from_str_radix(
                    s.get(1..).ok_or(Into::<String>::into("Invalid string"))?,
                    16,
                )
                .map_err(|_| Into::<String>::into("Invalid string"))?;
                Color::new(
                    ((number & 0x00ff0000) >> 8)
                        .try_into()
                        .map_err(|_| Into::<String>::into("Invalid string"))?,
                    ((number & 0x0000ff00) >> 4)
                        .try_into()
                        .map_err(|_| Into::<String>::into("Invalid string"))?,
                    (number & 0x000000ff)
                        .try_into()
                        .map_err(|_| Into::<String>::into("Invalid string"))?,
                    ((number & 0xff000000) >> 12)
                        .try_into()
                        .map_err(|_| Into::<String>::into("Invalid string"))?,
                )
            }
            _ => return Err(Into::<String>::into("Invalid string")),
        })
    }
}

#[derive(Parser)]
pub struct Config {
    #[arg(value_parser=|v:&str| v.parse::<Color>())]
    background_color: Color,
}

impl Config {
    pub fn background_color(&self) -> &Color {
        &self.background_color
    }
}
