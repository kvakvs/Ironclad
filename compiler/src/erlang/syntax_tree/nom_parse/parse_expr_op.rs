//! Parse an operator for binary or unary expressions
#![allow(missing_docs)]

use nom::{combinator, branch, character, bytes::complete::{tag}};

use crate::erlang::syntax_tree::erl_op::{ErlBinaryOp, ErlUnaryOp};

pub fn unop_catch(input: &str) -> nom::IResult<&str, ErlUnaryOp> {
  combinator::map(tag("catch"), |_| ErlUnaryOp::Catch)(input)
}

pub fn unop_not(input: &str) -> nom::IResult<&str, ErlUnaryOp> {
  combinator::map(tag("not"), |_| ErlUnaryOp::Not)(input)
}

pub fn unop_bnot(input: &str) -> nom::IResult<&str, ErlUnaryOp> {
  combinator::map(tag("bnot"), |_| ErlUnaryOp::BinaryNot)(input)
}

pub fn unop_positive(input: &str) -> nom::IResult<&str, ErlUnaryOp> {
  combinator::map(character::complete::char('+'), |_| ErlUnaryOp::Positive)(input)
}

pub fn unop_negative(input: &str) -> nom::IResult<&str, ErlUnaryOp> {
  combinator::map(character::complete::char('-'), |_| ErlUnaryOp::Negative)(input)
}

pub fn binop_floatdiv(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(character::complete::char('/'), |_| ErlBinaryOp::Div)(input)
}

pub fn binop_intdiv(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("div"), |_| ErlBinaryOp::IntegerDiv)(input)
}

pub fn binop_bang(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(character::complete::char('!'), |_| ErlBinaryOp::Bang)(input)
}

pub fn binop_multiply(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(character::complete::char('*'), |_| ErlBinaryOp::Mul)(input)
}

pub fn binop_add(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(character::complete::char('+'), |_| ErlBinaryOp::Add)(input)
}

pub fn binop_subtract(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(character::complete::char('-'), |_| ErlBinaryOp::Sub)(input)
}

pub fn binop_list_append(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("++"), |_| ErlBinaryOp::ListAppend)(input)
}

pub fn binop_list_subtract(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("--"), |_| ErlBinaryOp::ListSubtract)(input)
}

pub fn binop_rem(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("rem"), |_| ErlBinaryOp::Remainder)(input)
}

pub fn binop_and(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("and"), |_| ErlBinaryOp::And)(input)
}

pub fn binop_band(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("band"), |_| ErlBinaryOp::BinaryAnd)(input)
}

pub fn binop_or(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("or"), |_| ErlBinaryOp::Or)(input)
}

pub fn binop_orelse(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("orelse"), |_| ErlBinaryOp::OrElse)(input)
}

pub fn binop_andalso(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("andalso"), |_| ErlBinaryOp::AndAlso)(input)
}

pub fn binop_bor(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("bor"), |_| ErlBinaryOp::BinaryOr)(input)
}

pub fn binop_xor(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("xor"), |_| ErlBinaryOp::Xor)(input)
}

pub fn binop_bxor(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("bxor"), |_| ErlBinaryOp::BinaryXor)(input)
}

pub fn binop_bsl(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("bsl"), |_| ErlBinaryOp::BinaryShiftLeft)(input)
}

pub fn binop_bsr(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("bsr"), |_| ErlBinaryOp::BinaryShiftRight)(input)
}

pub fn binop_equals(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("=="), |_| ErlBinaryOp::Eq)(input)
}

pub fn binop_hard_equals(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("=:="), |_| ErlBinaryOp::HardEq)(input)
}

pub fn binop_not_equals(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("/="), |_| ErlBinaryOp::NotEq)(input)
}

pub fn binop_hard_not_equals(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("=/="), |_| ErlBinaryOp::HardNotEq)(input)
}

pub fn binop_less(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(character::complete::char('<'), |_| ErlBinaryOp::Less)(input)
}

pub fn binop_less_eq(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("=<"), |_| ErlBinaryOp::LessEq)(input)
}

pub fn binop_greater(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(character::complete::char('>'), |_| ErlBinaryOp::Greater)(input)
}

pub fn binop_greater_eq(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag(">="), |_| ErlBinaryOp::GreaterEq)(input)
}

pub fn binop_match(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag("="), |_| ErlBinaryOp::Match)(input)
}

pub fn binop_comma(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag(","), |_| ErlBinaryOp::Comma)(input)
}

pub fn binop_semicolon(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  combinator::map(tag(";"), |_| ErlBinaryOp::Semicolon)(input)
}

fn parse_binop(input: &str) -> nom::IResult<&str, ErlBinaryOp> {
  branch::alt((
    branch::alt((binop_floatdiv, binop_multiply, binop_intdiv, binop_rem,
                 binop_band, binop_and,
                 binop_list_append, binop_list_subtract,
                 binop_add, binop_subtract)),
    branch::alt((binop_bor, binop_bxor, binop_or, binop_xor,
                 binop_bsl, binop_bsr,
                 binop_equals, binop_not_equals, binop_hard_equals, binop_hard_not_equals,
                 binop_less_eq, binop_greater_eq, binop_less, binop_greater,
                 binop_match)),
    branch::alt((binop_comma, binop_semicolon,
                 binop_andalso, binop_orelse)),
  ))(input)
}