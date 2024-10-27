use crate::cli::Color;

#[derive(PartialEq, Debug, Default)]
pub struct StyledString {
    content: Vec<StyledStringPart>,
}

impl StyledString {
    pub fn new(content: Vec<StyledStringPart>) -> Self {
        Self { content }
    }

    pub fn content(&self) -> &[StyledStringPart] {
        &self.content
    }

    pub fn into_content(self) -> Vec<StyledStringPart> {
        self.content
    }
}

#[derive(PartialEq, Debug)]
pub enum StyledStringPart {
    String(String),
    Style(Style),
}

#[derive(PartialEq, Debug)]
pub struct Style {
    foreground_color: Option<Color>,
    background_color: Option<Color>,
}

impl Style {
    pub fn new(foreground_color: Option<Color>, background_color: Option<Color>) -> Self {
        Self {
            foreground_color,
            background_color,
        }
    }

    pub fn foreground_color(&self) -> Option<Color> {
        self.foreground_color
    }

    pub fn background_color(&self) -> Option<Color> {
        self.background_color
    }
}

peg::parser! {
    grammar styled_string() for str {
        rule color() -> crate::cli::Color
        = n:['0'..='9'|'A'..='F'|'a'..='f']*<3,8> {?
            format!("#{}", n.iter().collect::<String>()).parse().map_err(|_| "Invalid string")
        }
        rule formatting_block() -> Style
            = "%{B" c:color() "}" {Style{foreground_color:None, background_color:Some(c)}}
            / "%{F" c:color() "}" {Style{foreground_color:Some(c), background_color:None}}
        rule part() -> StyledStringPart
            = f:formatting_block() {StyledStringPart::Style(f)}
            / s:([^'%']+) {StyledStringPart::String(s.iter().collect())}
        pub rule string() -> StyledString
            = c:(part()*) {StyledString{content:c}}
    }
}

impl std::str::FromStr for StyledString {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        styled_string::string(s).map_err(|v| {
            format!(
                "Formatting failed at {}, expected {}",
                v.location, v.expected
            )
        })
    }
}
