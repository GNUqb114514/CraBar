#[derive(PartialEq, Debug, Default)]
pub struct StyledString {
    content: Vec<StyledStringPart>,
}

impl StyledString {
    pub fn into_content(self) -> Vec<StyledStringPart> {
        self.content
    }
}

#[derive(PartialEq, Debug)]
pub struct Action {
    button: u8,
    cmd: String,
}

impl Action {
    pub fn into_tuple(self) -> (u8, String) {
        (self.button, self.cmd)
    }
}

#[derive(PartialEq, Debug)]
pub enum StyledStringPart {
    String(String),
    Style(Style),
    Action(Action),
    ActionEnd,
    Swap,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Color {
    Default,
    Now,
    New(crate::cli::Color),
}

impl Color {
    pub fn into_color(
        self,
        default: crate::cli::Color,
        now: crate::cli::Color,
    ) -> crate::cli::Color {
        match self {
            Self::Default => default,
            Self::Now => now,
            Self::New(a) => a,
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct Style {
    foreground_color: Color,
    background_color: Color,
}

impl Style {
    pub fn foreground_color(&self) -> Color {
        self.foreground_color
    }

    pub fn background_color(&self) -> Color {
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
            = "%{B" c:color() "}" {Style{foreground_color:Color::Now, background_color:Color::New(c)}}
            / "%{F" c:color() "}" {Style{foreground_color:Color::New(c), background_color:Color::Now}}
            / "%{B-}" {Style{foreground_color:Color::Now, background_color:Color::Default}}
            / "%{F-}" {Style{background_color:Color::Now, foreground_color:Color::Default}}
        rule action() -> StyledStringPart
            = "%{A" button:(['1'..='5']?) ":" cmd:([^':']+) ":}" {?
                Ok(StyledStringPart::Action(Action{
                    button:button.unwrap_or('1') as u8 - '0' as u8, cmd:cmd.iter().collect()
                }))
            }
            / "%{A}" {StyledStringPart::ActionEnd}
            / "%{R}" {StyledStringPart::Swap}
        rule part() -> StyledStringPart
            = f:formatting_block() {StyledStringPart::Style(f)}
            / a:action() {a}
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
