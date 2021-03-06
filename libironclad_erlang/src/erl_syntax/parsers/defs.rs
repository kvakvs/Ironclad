//! Definitions for parser

use crate::erl_syntax::parsers::parser_error::ErlParserError;
use crate::erl_syntax::parsers::parser_input::ParserInput;

/// Generic return value from a Nom parser which takes a `ParserInput` and returns `Out`.
/// Contains no scope field.
pub type ParserResult<'a, Out> = nom::IResult<ParserInput<'a>, Out, ErlParserError<'a>>;

/// Use when Nom's char is imported and this confuses the editor
pub type Char = char;

// /// Return value from a Nom parser which takes &str and returns `Vec<AstNode>`
// pub type VecAstParserResult<'a> = ParserResult<'a, Vec<AstNode>>;

/// Return value from a Nom parser which takes &str and returns `&str`
pub type StrSliceParserResult<'a> = ParserResult<'a, ParserInput<'a>>;
