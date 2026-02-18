use serenity::all::ShardId;
use utils::{Formatter, Parser};

const NOT_AVAILABLE: &str = "`n/a`";

fn new_parser() -> Parser {
    Parser::new(ShardId(0))
}

#[test]
fn format_returns_input_without_placeholders() {
    let mut parser = new_parser();
    let input = "plain text";
    assert_eq!(input.format(&mut parser), input);
}

#[test]
fn format_unknown_section_returns_not_available() {
    let mut parser = new_parser();
    let input = "Value: {unknown.key}";
    let expected = format!("Value: {}", NOT_AVAILABLE);
    assert_eq!(input.format(&mut parser), expected);
}

#[test]
fn format_missing_user_returns_not_available() {
    let mut parser = new_parser();
    let input = "{user.id}";
    assert_eq!(input.format(&mut parser), NOT_AVAILABLE);
}

#[test]
fn format_multiple_unknown_placeholders() {
    let mut parser = new_parser();
    let input = "A {foo.bar} B {baz.qux}";
    let expected = format!("A {} B {}", NOT_AVAILABLE, NOT_AVAILABLE);
    assert_eq!(input.format(&mut parser), expected);
}

#[test]
fn format_time_now_text_is_non_empty() {
    let mut parser = new_parser();
    let output = "{time.now_text}".format(&mut parser);
    assert!(!output.is_empty());
    assert!(!output.contains('{'));
    assert!(!output.contains('}'));
}
