// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Eric Naville

//! Lexer für den f64-Ausdrucks-Evaluator.
//!
//! Erkennt Zahlen (inkl. optionalem Exponent), ASCII-Identifier
//! (Buchstaben + `_`, fortgesetzt durch Buchstaben/Ziffern/`_`),
//! Klammern und die Operatoren `+ - * / ^ %`. Whitespace wird
//! übersprungen. Alle Tokens sind ASCII — die Custom-Operatoren
//! `⊕`, `√`, `log` werden bereits in `resolve_custom_operators`
//! aufgelöst, bevor der String hier ankommt.

use core::fmt;

#[derive(Clone, Debug)]
pub(super) enum FTok {
    Num(f64),
    Ident(String),
    LParen,
    RParen,
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Percent,
}

#[derive(Debug)]
pub(super) struct ParseError;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parse error")
    }
}

pub(super) fn tokenize(src: &str) -> Result<Vec<FTok>, ParseError> {
    let bytes = src.as_bytes();
    let mut pos = 0usize;
    let mut out = Vec::new();

    while pos < bytes.len() {
        let c = bytes[pos];
        // Whitespace
        if c == b' ' || c == b'\t' || c == b'\n' || c == b'\r' {
            pos += 1;
            continue;
        }
        // Zahl-Literal (auch mit führendem `.`)
        if c.is_ascii_digit() || c == b'.' {
            let start = pos;
            while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                pos += 1;
            }
            if pos < bytes.len() && bytes[pos] == b'.' {
                pos += 1;
                while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                    pos += 1;
                }
            }
            // Optionaler Exponent
            if pos < bytes.len() && (bytes[pos] == b'e' || bytes[pos] == b'E') {
                pos += 1;
                if pos < bytes.len() && (bytes[pos] == b'+' || bytes[pos] == b'-') {
                    pos += 1;
                }
                while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                    pos += 1;
                }
            }
            let text = &src[start..pos];
            let n: f64 = text.parse().map_err(|_| ParseError)?;
            out.push(FTok::Num(n));
            continue;
        }
        // Identifier
        if c.is_ascii_alphabetic() || c == b'_' {
            let start = pos;
            pos += 1;
            while pos < bytes.len() && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_') {
                pos += 1;
            }
            let name = &src[start..pos];
            out.push(FTok::Ident(name.to_string()));
            continue;
        }
        // Einzelzeichen-Operatoren
        let tok = match c {
            b'(' => FTok::LParen,
            b')' => FTok::RParen,
            b'+' => FTok::Plus,
            b'-' => FTok::Minus,
            b'*' => FTok::Star,
            b'/' => FTok::Slash,
            b'^' => FTok::Caret,
            b'%' => FTok::Percent,
            _ => return Err(ParseError),
        };
        out.push(tok);
        pos += 1;
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple_arithmetic() {
        let t = tokenize("1 + 2").unwrap();
        assert!(matches!(t[0], FTok::Num(_)));
        assert!(matches!(t[1], FTok::Plus));
        assert!(matches!(t[2], FTok::Num(_)));
    }

    #[test]
    fn tokenize_function_call() {
        let t = tokenize("sin(0)").unwrap();
        assert!(matches!(t[0], FTok::Ident(ref s) if s == "sin"));
        assert!(matches!(t[1], FTok::LParen));
        assert!(matches!(t[2], FTok::Num(_)));
        assert!(matches!(t[3], FTok::RParen));
    }

    #[test]
    fn tokenize_decimal_and_exponent() {
        let t = tokenize("1.5e10").unwrap();
        match &t[0] {
            FTok::Num(n) => assert!((n - 1.5e10).abs() < 1e-6),
            _ => panic!("expected number token"),
        }
    }

    #[test]
    fn tokenize_unknown_char_errors() {
        assert!(tokenize("1 $ 2").is_err());
    }
}
