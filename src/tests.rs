extern crate std;

use crate::*;

#[track_caller]
fn check(s: &str, expected: &[Item]) {
    let value: std::vec::Vec<_> = Parser::new(s).collect();
    assert_eq!(value, expected);
}

#[track_caller]
fn check_err(s: &str, line: usize) {
    for (index, item) in Parser::new(s).enumerate() {
        if index == line {
            assert!(matches!(item, Item::Error(_)));
        }
    }
}

#[test]
fn test_eos() {
    check(
        "\r\n[SECTION]",
        &[
            Item::Blank { raw: "" },
            Item::SectionEnd,
            Item::Section {
                name: "SECTION",
                raw: "[SECTION]",
            },
            Item::SectionEnd,
        ],
    );
    check(
        "\r\n[SECTION]\n",
        &[
            Item::Blank { raw: "" },
            Item::SectionEnd,
            Item::Section {
                name: "SECTION",
                raw: "[SECTION]",
            },
            Item::SectionEnd,
        ],
    );
    check(
        "\r\n;comment",
        &[
            Item::Blank { raw: "" },
            Item::Comment { raw: ";comment" },
            Item::SectionEnd,
        ],
    );
    check(
        "\r\n;comment\n",
        &[
            Item::Blank { raw: "" },
            Item::Comment { raw: ";comment" },
            Item::SectionEnd,
        ],
    );
    check(
        "\r\nKey=Value",
        &[
            Item::Blank { raw: "" },
            Item::Property {
                key: "Key",
                val: Some("Value"),
                raw: "Key=Value",
            },
            Item::SectionEnd,
        ],
    );
    check(
        "\r\nKey=Value\n",
        &[
            Item::Blank { raw: "" },
            Item::Property {
                key: "Key",
                val: Some("Value"),
                raw: "Key=Value",
            },
            Item::SectionEnd,
        ],
    );
    check(
        "\r\nKey=Value\r",
        &[
            Item::Blank { raw: "" },
            Item::Property {
                key: "Key",
                val: Some("Value"),
                raw: "Key=Value",
            },
            Item::SectionEnd,
        ],
    );
    check(
        "\r\nKey=Value\r\n",
        &[
            Item::Blank { raw: "" },
            Item::Property {
                key: "Key",
                val: Some("Value"),
                raw: "Key=Value",
            },
            Item::SectionEnd,
        ],
    );
    check(
        "\r\nAction",
        &[
            Item::Blank { raw: "" },
            Item::Property {
                key: "Action",
                val: None,
                raw: "Action",
            },
            Item::SectionEnd,
        ],
    );
    check(
        "\r\nAction\n",
        &[
            Item::Blank { raw: "" },
            Item::Property {
                key: "Action",
                val: None,
                raw: "Action",
            },
            Item::SectionEnd,
        ],
    );
    check(
        "\r\nAction\r",
        &[
            Item::Blank { raw: "" },
            Item::Property {
                key: "Action",
                val: None,
                raw: "Action",
            },
            Item::SectionEnd,
        ],
    );
    check(
        "\r\nAction\r\n",
        &[
            Item::Blank { raw: "" },
            Item::Property {
                key: "Action",
                val: None,
                raw: "Action",
            },
            Item::SectionEnd,
        ],
    );
}

#[test]
fn test_empty_strings() {
    check(
        "[]\n=\r\n = \n;\n \r= \r\n =\n=",
        &[
            Item::SectionEnd,
            Item::Section {
                name: "",
                raw: "[]",
            },
            Item::Property {
                key: "",
                val: Some(""),
                raw: "=",
            },
            Item::Property {
                key: "",
                val: Some(""),
                raw: " = ",
            },
            Item::Comment { raw: ";" },
            Item::Blank { raw: " " },
            Item::Property {
                key: "",
                val: Some(""),
                raw: "= ",
            },
            Item::Property {
                key: "",
                val: Some(""),
                raw: " =",
            },
            Item::Property {
                key: "",
                val: Some(""),
                raw: "=",
            },
            Item::SectionEnd,
        ],
    );
}

#[test]
fn test_syntax_errors() {
    check_err("[foo] ", 1);
    check_err("[foo] \r", 1);
    check_err("[foo] \n", 1);
    check_err("[foo", 1);
    check_err("[foo\r", 1);
    check_err("[foo\n", 1);
    check_err("[", 1);
    check_err("[\r", 1);
    check_err("[\n", 1);
    check_err("[foo]\n[", 3);
}

#[test]
fn test_blank_lines() {
    check(
        "\n\r\n\r",
        &[
            Item::Blank { raw: "" },
            Item::Blank { raw: "" },
            Item::Blank { raw: "" },
            Item::SectionEnd,
        ],
    );
    check(
        "\r\r\n\r",
        &[
            Item::Blank { raw: "" },
            Item::Blank { raw: "" },
            Item::Blank { raw: "" },
            Item::SectionEnd,
        ],
    );
    check(
        "\r\r\r\n",
        &[
            Item::Blank { raw: "" },
            Item::Blank { raw: "" },
            Item::Blank { raw: "" },
            Item::SectionEnd,
        ],
    );
}

#[test]
fn test_terminates() {
    // Ensure syntax errors advance the internal parser state
    check(
        "[\n[] \r\n",
        &[
            Item::SectionEnd,
            Item::Error("["),
            Item::SectionEnd,
            Item::Error("[] "),
            Item::SectionEnd,
        ],
    );
    for _ in Parser::new("[") {}
    for _ in Parser::new("[] ") {}
}
