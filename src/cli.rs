use clap::Parser;

#[derive(Clone, Debug)]
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

impl From<Color> for clap::builder::OsStr {
    fn from(value: Color) -> Self {
        let str: &str = format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            value.r, value.g, value.b, value.a
        )
        .leak();
        str.into()
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{:02x}{:02x}{:02x}{:02x}",
            self.r, self.g, self.b, self.a
        )
    }
}

impl core::str::FromStr for Color {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn inner(s: &str) -> Result<Color, ()> {
            if s.get(0..0).is_some_and(|v| v == "#") {
                return Err(());
            }
            Ok(match s.len() {
                4 => {
                    let number = u32::from_str_radix(s.get(1..).ok_or(())?, 16).map_err(|_| ())?;
                    Color::new(
                        ((number & 0xf00) >> 4).try_into().map_err(|_| ())?,
                        ((number & 0x0f0) >> 0).try_into().map_err(|_| ())?,
                        ((number & 0x00f) << 4).try_into().map_err(|_| ())?,
                        0xff,
                    )
                }
                5 => {
                    let number = u32::from_str_radix(s.get(1..).ok_or(())?, 16).map_err(|_| ())?;
                    Color::new(
                        ((number & 0xf000) >> 4).try_into().map_err(|_| ())?,
                        ((number & 0x0f00) >> 0).try_into().map_err(|_| ())?,
                        ((number & 0x00f0) << 4).try_into().map_err(|_| ())?,
                        ((number & 0xf000) >> 8).try_into().map_err(|_| ())?,
                    )
                }
                7 => {
                    let number = u32::from_str_radix(s.get(1..).ok_or(())?, 16).map_err(|_| ())?;
                    Color::new(
                        ((number & 0xff0000) >> 16).try_into().map_err(|_| ())?,
                        ((number & 0x00ff00) >> 8).try_into().map_err(|_| ())?,
                        (number & 0x0000ff).try_into().map_err(|_| ())?,
                        0xff,
                    )
                }
                9 => {
                    let number = u32::from_str_radix(s.get(1..).ok_or(())?, 16).map_err(|_| ())?;
                    Color::new(
                        ((number & 0xff000000) >> 16).try_into().map_err(|_| ())?,
                        ((number & 0x00ff0000) >> 8).try_into().map_err(|_| ())?,
                        (number & 0x0000ff00).try_into().map_err(|_| ())?,
                        ((number & 0x000000ff) >> 12).try_into().map_err(|_| ())?,
                    )
                }
                _ => return Err(()),
            })
        }
        inner(s).map_err(|_| format!("Invalid string: {}", s))
    }
}

#[derive(Parser)]
pub struct Config {
    #[arg(value_parser=|v:&str| v.parse::<Color>(), default_value="#ffffff")]
    background_color: Color,
}

impl Config {
    pub fn background_color(&self) -> &Color {
        &self.background_color
    }
}
