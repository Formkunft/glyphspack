use anyhow::{Context, Result};
use pest::Parser;
use pest::error::Error;
use pest::iterators::Pair;
use pest_derive::Parser;
use std::borrow::Cow;
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Parser)]
#[grammar = "grammar/plist.pest"]
struct PlistParser;

#[derive(Clone, Copy)]
pub enum Root {
    Dict,
    Array,
}

#[derive(Debug)]
pub struct Slice<'a> {
    pub value: Value<'a>,
    pub code: &'a str,
}

#[derive(Debug)]
pub enum Value<'a> {
    Dict(Vec<(Cow<'a, str>, Slice<'a>, &'a str)>),
    Array(Vec<Slice<'a>>),
    String(Cow<'a, str>),
}

pub fn parse(root: Root, code: &str) -> Result<Slice<'_>, Error<Rule>> {
    fn parse_string(pair: Pair<Rule>) -> Cow<str> {
        match pair.as_rule() {
            Rule::string_quoted => {
                let contents = pair.into_inner().next().unwrap().as_str();
                if contents.contains('\\') {
                    let mut result = String::with_capacity(contents.len());
                    let mut chars = contents.chars();
                    while let Some(c) = chars.next() {
                        if c == '\\' {
                            match chars.next() {
                                Some('"') => result.push('"'),
                                Some('/') => result.push('/'),
                                Some('n') => result.push('\n'),
                                Some('r') => result.push('\r'),
                                Some('t') => result.push('\t'),
                                Some('b') => result.push('\u{08}'),
                                Some('f') => result.push('\u{0C}'),
                                Some('0') => {
                                    let d1 = chars.next().unwrap_or('0') as u8 - b'0';
                                    let d2 = chars.next().unwrap_or('0') as u8 - b'0';
                                    result.push((d1 * 8 + d2) as char);
                                }
                                Some('\\') | None => result.push('\\'),
                                Some(other) => {
                                    result.push('\\');
                                    result.push(other);
                                }
                            }
                        } else {
                            result.push(c);
                        }
                    }
                    Cow::Owned(result)
                } else {
                    Cow::Borrowed(contents)
                }
            }
            Rule::string_unquoted => Cow::Borrowed(pair.as_str()),
            _ => unreachable!(),
        }
    }

    fn parse_slice(pair: Pair<Rule>) -> Slice {
        let rule = pair.as_rule();
        let mut pairs = pair.into_inner();

        match rule {
            Rule::dict => Slice {
                code: pairs.as_str(),
                value: Value::Dict({
                    pairs
                        .map(|pair| {
                            let code = pair.as_str();
                            let mut inner_rules = pair.into_inner();
                            let key = parse_string(
                                inner_rules.next().unwrap().into_inner().next().unwrap(),
                            );
                            let value = parse_slice(inner_rules.next().unwrap());
                            (key, value, code)
                        })
                        .collect()
                }),
            },
            Rule::array => Slice {
                code: pairs.as_str(),
                value: Value::Array(pairs.map(parse_slice).collect()),
            },
            Rule::string => Slice {
                code: pairs.as_str(),
                value: Value::String(parse_string(pairs.next().unwrap())),
            },
            Rule::value => parse_slice(pairs.next().unwrap()),
            _ => unreachable!(),
        }
    }

    let rule = match root {
        Root::Dict => Rule::dict,
        Root::Array => Rule::array,
    };
    let plist = PlistParser::parse(rule, code)?.next().unwrap();

    Ok(parse_slice(plist))
}

pub fn write_dict_file(path: &Path, codes: &[&str]) -> Result<()> {
    let mut file =
        fs::File::create(path).with_context(|| format!("cannot create {}", path.display()))?;

    writeln!(file, "{{")?;

    for code in codes {
        writeln!(file, "{code}")?;
    }

    writeln!(file, "}}")?;

    Ok(())
}

pub fn write_array_file(path: &Path, codes: &[&str]) -> Result<()> {
    let mut file =
        fs::File::create(path).with_context(|| format!("cannot create {}", path.display()))?;

    writeln!(file, "(")?;

    let mut iter = codes.iter().peekable();

    while let Some(code) = iter.next() {
        if iter.peek().is_some() {
            writeln!(file, "{code},")?;
        } else {
            writeln!(file, "{code}")?;
        }
    }

    write!(file, ")")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_string_value(code: &str) -> String {
        let slice = parse(Root::Dict, code).unwrap();
        match slice.value {
            Value::Dict(pairs) => match &pairs[0].1.value {
                Value::String(s) => s.to_string(),
                _ => panic!("expected string value"),
            },
            _ => panic!("expected dict"),
        }
    }

    #[test]
    fn parse_empty_dict() {
        let slice = parse(Root::Dict, "{}").unwrap();
        match slice.value {
            Value::Dict(pairs) => assert!(pairs.is_empty()),
            _ => panic!("expected dict"),
        }
    }

    #[test]
    fn parse_dict_with_entry() {
        let slice = parse(Root::Dict, "{key = value;}").unwrap();
        match slice.value {
            Value::Dict(pairs) => {
                assert_eq!(pairs.len(), 1);
                assert_eq!(pairs[0].0.as_ref(), "key");
            }
            _ => panic!("expected dict"),
        }
    }

    #[test]
    fn parse_nested_dict() {
        let slice = parse(Root::Dict, "{outer = {inner = val;};}").unwrap();
        match slice.value {
            Value::Dict(pairs) => {
                assert_eq!(pairs[0].0.as_ref(), "outer");
                match &pairs[0].1.value {
                    Value::Dict(inner) => assert_eq!(inner[0].0.as_ref(), "inner"),
                    _ => panic!("expected inner dict"),
                }
            }
            _ => panic!("expected dict"),
        }
    }

    #[test]
    fn parse_empty_array() {
        let slice = parse(Root::Array, "()").unwrap();
        match slice.value {
            Value::Array(items) => assert!(items.is_empty()),
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn parse_array_with_items() {
        let slice = parse(Root::Array, "(a, b, c)").unwrap();
        match slice.value {
            Value::Array(items) => assert_eq!(items.len(), 3),
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn parse_array_trailing_comma() {
        let slice = parse(Root::Array, "(a, b,)").unwrap();
        match slice.value {
            Value::Array(items) => assert_eq!(items.len(), 2),
            _ => panic!("expected array"),
        }
    }

    #[test]
    fn parse_unquoted_string() {
        let val = parse_string_value("{k = hello;}");
        assert_eq!(val, "hello");
    }

    #[test]
    fn parse_quoted_string() {
        let val = parse_string_value(r#"{k = "hello world";}"#);
        assert_eq!(val, "hello world");
    }

    #[test]
    fn parse_escape_newline() {
        let val = parse_string_value(r#"{k = "line1\nline2";}"#);
        assert_eq!(val, "line1\nline2");
    }

    #[test]
    fn parse_escape_tab() {
        let val = parse_string_value(r#"{k = "col1\tcol2";}"#);
        assert_eq!(val, "col1\tcol2");
    }

    #[test]
    fn parse_escape_backslash() {
        let val = parse_string_value(r#"{k = "a\\b";}"#);
        assert_eq!(val, "a\\b");
    }

    #[test]
    fn parse_escape_quote() {
        let val = parse_string_value(r#"{k = "say \"hello\"";}"#);
        assert_eq!(val, "say \"hello\"");
    }

    #[test]
    fn parse_no_escape_borrows() {
        let code = r#"{k = "no escapes here";}"#;
        let slice = parse(Root::Dict, code).unwrap();
        match slice.value {
            Value::Dict(pairs) => match &pairs[0].1.value {
                Value::String(s) => assert!(matches!(s, Cow::Borrowed(_))),
                _ => panic!("expected string"),
            },
            _ => panic!("expected dict"),
        }
    }

    #[test]
    fn parse_with_escape_owns() {
        let code = r#"{k = "has\nnewline";}"#;
        let slice = parse(Root::Dict, code).unwrap();
        match slice.value {
            Value::Dict(pairs) => match &pairs[0].1.value {
                Value::String(s) => assert!(matches!(s, Cow::Owned(_))),
                _ => panic!("expected string"),
            },
            _ => panic!("expected dict"),
        }
    }

    #[test]
    fn parse_comment() {
        let code = "{\n// this is a comment\nk = v;\n}";
        let slice = parse(Root::Dict, code).unwrap();
        match slice.value {
            Value::Dict(pairs) => assert_eq!(pairs.len(), 1),
            _ => panic!("expected dict"),
        }
    }

    #[test]
    fn parse_unquoted_with_dots() {
        let val = parse_string_value("{k = a.b.c;}");
        assert_eq!(val, "a.b.c");
    }

    #[test]
    fn parse_unquoted_with_hyphens() {
        let val = parse_string_value("{k = some-value;}");
        assert_eq!(val, "some-value");
    }

    #[test]
    fn parse_invalid_dict() {
        assert!(parse(Root::Dict, "not a dict").is_err());
    }

    #[test]
    fn write_dict_roundtrip() {
        let dir = std::env::temp_dir().join("plist_test_dict");
        let _ = fs::remove_file(&dir);
        write_dict_file(&dir, &["key = value;"]).unwrap();
        let content = fs::read_to_string(&dir).unwrap();
        assert_eq!(content, "{\nkey = value;\n}\n");
        let _ = fs::remove_file(&dir);
    }

    #[test]
    fn write_array_roundtrip() {
        let dir = std::env::temp_dir().join("plist_test_array");
        let _ = fs::remove_file(&dir);
        write_array_file(&dir, &["a", "b", "c"]).unwrap();
        let content = fs::read_to_string(&dir).unwrap();
        assert_eq!(content, "(\na,\nb,\nc\n)");
        let _ = fs::remove_file(&dir);
    }
}
