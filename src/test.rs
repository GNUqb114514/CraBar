#[test]
fn test_parse() {
    use crate::cli::Color;
    use crate::parse::*;
    let str = "test";
    let styled: StyledString = str.parse().unwrap();
    assert_eq!(
        styled,
        StyledString::new(vec![StyledStringPart::String("test".into())])
    );

    let str = "%{B0ff}";
    let styled: StyledString = str.parse().unwrap();
    assert_eq!(
        styled,
        StyledString::new(vec![StyledStringPart::Style(Style::new(
            None,
            Some(Color::new(0, 255, 255, 255))
        ))])
    );

    let str = "%{B0ff}test";
    let styled: StyledString = str.parse().unwrap();
    assert_eq!(
        styled,
        StyledString::new(vec![
            StyledStringPart::Style(Style::new(None, Some(Color::new(0, 255, 255, 255)))),
            StyledStringPart::String("test".into())
        ])
    );

    let str = "%{A:test:}test%{A}";
    let styled: StyledString = str.parse().unwrap();
    assert_eq!(
        styled,
        StyledString::new(vec![
            StyledStringPart::Action(Action::new(1, "test".into())),
            StyledStringPart::String("test".into()),
            StyledStringPart::ActionEnd,
        ])
    );
}
