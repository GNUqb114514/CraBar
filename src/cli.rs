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

impl Color {
    // Combine two colors by AlphaBlend.
    pub fn blend(&self, bg: &Self) -> Self {
        let aa: u32 = self.a.into();
        let ra: u32 = self.r.into();
        let ga: u32 = self.g.into();
        let ba: u32 = self.b.into();
        let ab: u32 = bg.a.into();
        let rb: u32 = bg.r.into();
        let gb: u32 = bg.g.into();
        let bb: u32 = bg.b.into();
        let ac: u8 = (255 as u32 - ((255 - aa) * (255 - ab) >> 8))
            .try_into()
            .unwrap();
        let rc: u8 = ((ra * (aa) >> 8) + (rb * (ab) * (1 - aa) >> 16))
            .try_into()
            .unwrap();
        let gc: u8 = ((ga * (aa) >> 8) + (gb * (ab) * (1 - aa) >> 16))
            .try_into()
            .unwrap();
        let bc: u8 = ((ba * (aa) >> 8) + (bb * (ab) * (1 - aa) >> 16))
            .try_into()
            .unwrap();
        Self {
            a: ac,
            r: rc,
            g: gc,
            b: bc,
        }
    }

    // Combine this with alpha
    pub fn with_alpha(&self, alpha: u8) -> Self {
        let orig: u32 = self.a.into();
        let new: u32 = alpha.into();
        let res: u8 = ((orig * new) >> 8).try_into().unwrap();
        Self { a: res, ..*self }
    }
}

impl Into<[u8; 4]> for &Color {
    /// Translate this to byte reprensation,
    /// in ARGB8888 format.
    fn into(self) -> [u8; 4] {
        [self.a, self.r, self.g, self.b]
    }
}

impl From<&[u8; 4]> for Color {
    /// Translate byte reprensation to this struct,
    /// in ARGB8888 format.
    fn from(value: &[u8; 4]) -> Self {
        Self {
            a: value[0],
            r: value[1],
            g: value[2],
            b: value[3],
        }
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
