// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Eric Naville

//! Recursive-Descent-Parser für den f64-Evaluator.
//!
//! Grammatik (von oben nach unten in Präzedenz):
//! ```text
//!   add_sub  := mul_div ([+−] mul_div)*
//!   mul_div  := unary ([*/%] unary)*
//!   unary    := [+−]* pow
//!   pow      := primary ('^' unary)?         (right-associative via unary)
//!   primary  := NUM | '(' add_sub ')' | IDENT ['(' add_sub ')']
//! ```
//!
//! IDENT ohne Klammern wird als Konstante (`pi`, `e`) aufgelöst. Mit
//! Klammern als Funktionsaufruf — die Funktionssemantik liegt in
//! [`interpret::apply_func`](super::interpret::apply_func).

use super::interpret::apply_func;
use super::lexer::{FTok, ParseError};
use crate::token::AngleMode;

pub(super) struct Parser<'a> {
    toks: &'a [FTok],
    pos: usize,
    angle_mode: AngleMode,
}

impl<'a> Parser<'a> {
    pub(super) fn new(toks: &'a [FTok], angle_mode: AngleMode) -> Self {
        Self {
            toks,
            pos: 0,
            angle_mode,
        }
    }

    pub(super) fn parse(&mut self) -> Result<f64, ParseError> {
        if self.toks.is_empty() {
            return Err(ParseError);
        }
        let v = self.add_sub()?;
        if self.pos != self.toks.len() {
            return Err(ParseError);
        }
        Ok(v)
    }

    fn peek(&self) -> Option<&FTok> {
        self.toks.get(self.pos)
    }

    fn add_sub(&mut self) -> Result<f64, ParseError> {
        let mut left = self.mul_div()?;
        loop {
            match self.peek() {
                Some(FTok::Plus) => {
                    self.pos += 1;
                    left += self.mul_div()?;
                }
                Some(FTok::Minus) => {
                    self.pos += 1;
                    left -= self.mul_div()?;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn mul_div(&mut self) -> Result<f64, ParseError> {
        let mut left = self.unary()?;
        loop {
            match self.peek() {
                Some(FTok::Star) => {
                    self.pos += 1;
                    left *= self.unary()?;
                }
                Some(FTok::Slash) => {
                    self.pos += 1;
                    left /= self.unary()?;
                }
                Some(FTok::Percent) => {
                    self.pos += 1;
                    left %= self.unary()?;
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<f64, ParseError> {
        match self.peek() {
            Some(FTok::Plus) => {
                self.pos += 1;
                self.unary()
            }
            Some(FTok::Minus) => {
                self.pos += 1;
                Ok(-self.unary()?)
            }
            _ => self.pow(),
        }
    }

    fn pow(&mut self) -> Result<f64, ParseError> {
        let base = self.primary()?;
        if matches!(self.peek(), Some(FTok::Caret)) {
            self.pos += 1;
            // right-associative via unary → so 2^-3 und 2^3^2 parsen korrekt
            let exp = self.unary()?;
            return Ok(base.powf(exp));
        }
        Ok(base)
    }

    fn primary(&mut self) -> Result<f64, ParseError> {
        let tok = self.peek().ok_or(ParseError)?;
        match tok {
            FTok::Num(n) => {
                let n = *n;
                self.pos += 1;
                Ok(n)
            }
            FTok::LParen => {
                self.pos += 1;
                let v = self.add_sub()?;
                if !matches!(self.peek(), Some(FTok::RParen)) {
                    return Err(ParseError);
                }
                self.pos += 1;
                Ok(v)
            }
            FTok::Ident(name) => {
                let name = name.clone();
                self.pos += 1;
                match name.as_str() {
                    "pi" => return Ok(std::f64::consts::PI),
                    "e" => return Ok(std::f64::consts::E),
                    _ => {}
                }
                if !matches!(self.peek(), Some(FTok::LParen)) {
                    return Err(ParseError);
                }
                self.pos += 1;
                let arg = self.add_sub()?;
                if !matches!(self.peek(), Some(FTok::RParen)) {
                    return Err(ParseError);
                }
                self.pos += 1;
                apply_func(&name, arg, self.angle_mode).ok_or(ParseError)
            }
            _ => Err(ParseError),
        }
    }
}
