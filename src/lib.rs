/*!
# Format preserving Ini streaming parser

Simple INI parser with the following features:

Features:
* Format-preserving (you can write out again and get identical result)
* Fast!
* Streaming
* `no_std` support

Caveats:
* The Display trait on [Item] does *not* preserve formatting, if this is
  something you want, make sure to use the `raw` attributes to extract
  the raw line instead.
* Newlines are not saved. It is up to the caller to keep track of the
  type of newline in use. Mixed newline (e.g. a mix of CR, CRLF and LF) is
  supported on loading, but not on saving.

## Examples

```
use ini_roundtrip as ini;

let document = "\
[SECTION]
;this is a comment
Key = Value  ";

let elements = [
    ini::Item::SectionEnd,
    ini::Item::Section{name: "SECTION", raw: "[SECTION]"},
    ini::Item::Comment{raw: ";this is a comment"},
    ini::Item::Property{key: "Key", val: Some("Value"), raw: "Key = Value  "},
    ini::Item::SectionEnd,
];

for (index, item) in ini::Parser::new(document).enumerate() {
    assert_eq!(item, elements[index]);
}
```

The `SectionEnd` pseudo element is returned before a new section and at the end of the document.
This helps processing sections after their properties finished parsing.

The parser is very much line-based, it will continue no matter what and return nonsense as an item:

```
use ini_roundtrip as ini;

let document = "\
[SECTION
nonsense";

let elements = [
    ini::Item::SectionEnd,
    ini::Item::Error("[SECTION"),
    ini::Item::Property{key: "nonsense", val: None, raw: "nonsense"},
    ini::Item::SectionEnd,
];

for (index, item) in ini::Parser::new(document).enumerate() {
    assert_eq!(item, elements[index]);
}
```

Lines starting with `[` but contain either no closing `]` or a closing `]` not followed by a newline are returned as [`Item::Error`].
Lines missing a `=` are returned as [`Item::Property`] with `None` value. See below for more details.

Format
------

INI is not a well specified format, this parser tries to make as little assumptions as possible but it does make decisions.

* Newline is either `"\r\n"`, `"\n"` or `"\r"`. It can be mixed in a single document but this is not recommended.
* Section header is `"[" section "]" newline`. `section` can be anything except contain newlines.
* Property is `key "=" value newline`. `key` and `value` can be anything except contain newlines.
* Comment is the raw line for lines starting with `;` or `#`
* Blank is just `newline`.

Padding whitespace is always trimmed, but the raw line is always stored as well.

No further processing of the input is done, eg. if escape sequences are necessary they must be processed by the caller.
*/

#![no_std]
#![warn(unreachable_pub)]
#![warn(clippy::doc_markdown)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::ptr_as_ptr)]
#![warn(clippy::redundant_closure_for_method_calls)]
#![warn(clippy::semicolon_if_nothing_returned)]

use core::{fmt, str};

/// All the routines here work only with and slice only at ascii characters
/// This means conversion between `&str` and `&[u8]` is a noop even when slicing
#[inline]
fn from_utf8(v: &[u8]) -> &str {
    #[cfg(not(debug_assertions))]
    return unsafe { str::from_utf8_unchecked(v) };
    #[cfg(debug_assertions)]
    return str::from_utf8(v).unwrap();
}

/// Trims ascii whitespace from the start and end of the string slice.
fn trim(s: &str) -> &str {
    s.trim_matches(|chr: char| chr.is_ascii_whitespace())
}

/// A parsed element of syntatic meaning
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Item<'a> {
    /// Syntax error.
    ///
    /// Section header element was malformed.
    /// Malformed section headers are defined by a line starting with `[` but not ending with `]`.
    ///
    /// ```
    /// assert_eq!(
    ///     ini_roundtrip::Parser::new("[Error").nth(1),
    ///     Some(ini_roundtrip::Item::Error("[Error")));
    /// ```
    Error(&'a str),

    /// Section header element.
    ///
    /// ```
    /// assert_eq!(
    ///     ini_roundtrip::Parser::new("[Section]").nth(1),
    ///     Some(ini_roundtrip::Item::Section{name: "Section", raw: "[Section]"}));
    /// ```
    Section {
        /// Trimmed name of the section
        name: &'a str,
        /// Raw line
        raw: &'a str,
    },

    /// End of section.
    ///
    /// Pseudo element emitted before a [`Section`](Item::Section) and at the end of the document.
    /// This helps processing sections after their properties finished parsing.
    ///
    /// ```
    /// assert_eq!(
    ///     ini_roundtrip::Parser::new("").next(),
    ///     Some(ini_roundtrip::Item::SectionEnd));
    /// ```
    SectionEnd,

    /// Property element.
    ///
    /// Key value must not contain `=`.
    ///
    /// The value is `None` if there is no `=`.
    ///
    /// ```
    /// assert_eq!(
    ///     ini_roundtrip::Parser::new("Key=Value").next(),
    ///     Some(ini_roundtrip::Item::Property{key: "Key", val: Some("Value"), raw: "Key=Value"}));
    /// assert_eq!(
    ///     ini_roundtrip::Parser::new("Key").next(),
    ///     Some(ini_roundtrip::Item::Property{key: "Key", val: None, raw: "Key"}));
    /// ```
    Property {
        /// Trimmed key
        key: &'a str,
        /// Trimmed value (if any)
        val: Option<&'a str>,
        /// Raw line
        raw: &'a str,
    },

    /// Comment.
    ///
    /// ```
    /// assert_eq!(
    ///     ini_roundtrip::Parser::new(";comment").next(),
    ///     Some(ini_roundtrip::Item::Comment{raw: ";comment"}));
    /// ```
    Comment {
        /// Raw line
        raw: &'a str,
    },

    /// Blank line.
    ///
    /// Allows faithful reproduction of the whole ini document including blank lines.
    ///
    /// ```
    /// assert_eq!(
    ///     ini_roundtrip::Parser::new("\n").next(),
    ///     Some(ini_roundtrip::Item::Blank{raw: ""}));
    /// ```
    Blank {
        /// Raw line
        raw: &'a str,
    },
}

impl<'a> fmt::Display for Item<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Item::Error(error) => writeln!(f, "{error}"),
            Item::Section { name, raw: _ } => writeln!(f, "[{name}]"),
            Item::SectionEnd => Ok(()),
            Item::Property {
                key,
                val: Some(value),
                raw: _,
            } => writeln!(f, "{key}={value}"),
            Item::Property {
                key,
                val: None,
                raw: _,
            } => writeln!(f, "{key}"),
            Item::Comment { raw: comment } => writeln!(f, ";{comment}"),
            Item::Blank { raw: _ } => f.write_str("\n"),
        }
    }
}

/// Ini streaming parser.
///
/// The whole document must be available before parsing starts.
/// The parser then returns each element as it is being parsed.
///
/// See [`crate`] documentation for more information.
#[derive(Clone, Debug)]
pub struct Parser<'a> {
    line: u32,
    section_ended: bool,
    state: &'a [u8],
}

impl<'a> Parser<'a> {
    /// Constructs a new `Parser` instance.
    #[inline]
    pub const fn new(s: &'a str) -> Parser<'a> {
        let state = s.as_bytes();
        Parser {
            line: 0,
            section_ended: false,
            state,
        }
    }

    /// Returns the line number the parser is currently at.
    #[inline]
    pub const fn line(&self) -> u32 {
        self.line
    }

    /// Returns the remainder of the input string.
    #[inline]
    pub fn remainder(&self) -> &'a str {
        from_utf8(self.state)
    }

    #[inline]
    fn skip_ln(&mut self, mut s: &'a [u8]) {
        if !s.is_empty() {
            if s[0] == b'\r' {
                s = &s[1..];
            }
            if !s.is_empty() && s[0] == b'\n' {
                s = &s[1..];
            }
            self.line += 1;
        }
        self.state = s;
    }

    fn get_line_and_advance(&mut self, s: &'a [u8]) -> &'a str {
        let i = parse::find_nl(s);
        let line = from_utf8(&s[..i]);
        self.skip_ln(&s[i..]);
        line
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Item<'a>;

    fn next(&mut self) -> Option<Item<'a>> {
        let s = self.state;

        match s.first().copied() {
            // Terminal case
            None => {
                if self.section_ended {
                    None
                } else {
                    self.section_ended = true;
                    Some(Item::SectionEnd)
                }
            }
            // Blank
            Some(b'\r' | b'\n') => {
                let line = self.get_line_and_advance(s);
                Some(Item::Blank { raw: line })
            }
            // Comment
            Some(b';' | b'#') => {
                let line = self.get_line_and_advance(s);
                Some(Item::Comment { raw: line })
            }
            // Section
            Some(b'[') => {
                if self.section_ended {
                    self.section_ended = false;
                    let i = parse::find_nl(s);
                    if s[i - 1] != b']' {
                        let error = from_utf8(&s[..i]);
                        self.skip_ln(&s[i..]);
                        return Some(Item::Error(error));
                    }
                    let section = from_utf8(&s[1..i - 1]);
                    let section = trim(section);
                    self.skip_ln(&s[i..]);
                    Some(Item::Section {
                        name: section,
                        raw: from_utf8(&s[..i]),
                    })
                } else {
                    self.section_ended = true;
                    Some(Item::SectionEnd)
                }
            }
            // Property
            _ => {
                let eol_or_eq = parse::find_nl_chr(s, b'=');
                let key = from_utf8(&s[..eol_or_eq]);
                let key = trim(key);
                if s.get(eol_or_eq) != Some(&b'=') {
                    // Key only case
                    self.skip_ln(&s[eol_or_eq..]);
                    if key.is_empty() {
                        return Some(Item::Blank {
                            raw: from_utf8(&s[..eol_or_eq]),
                        });
                    }
                    return Some(Item::Property {
                        key,
                        val: None,
                        raw: from_utf8(&s[..eol_or_eq]),
                    });
                } else {
                    // Key + value case
                    let val_start = &s[eol_or_eq + 1..];

                    let i = parse::find_nl(val_start);
                    let value = from_utf8(&val_start[..i]);
                    let value = trim(value);

                    self.skip_ln(&val_start[i..]);

                    Some(Item::Property {
                        key,
                        val: Some(value),
                        raw: from_utf8(&s[..eol_or_eq + i + 1]),
                    })
                }
            }
        }
    }
}

impl<'a> core::iter::FusedIterator for Parser<'a> {}

mod parse;
#[cfg(test)]
mod tests;
