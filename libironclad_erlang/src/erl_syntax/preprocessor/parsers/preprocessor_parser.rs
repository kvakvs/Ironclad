//! Groups type definitions shared by all preprocessor parse modules
use crate::erl_syntax::erl_ast::AstNode;
use crate::erl_syntax::parsers::defs::{ParserInput, ParserResult};
use crate::erl_syntax::parsers::misc::{
  comma_tag, match_dash_tag, newline_or_eof, par_close_tag, par_open_tag, period_newline_tag,
  period_tag, ws_before, ws_before_mut,
};
use crate::erl_syntax::parsers::parse_strings::parse_str::parse_doublequot_string;
use crate::erl_syntax::preprocessor::ast::PreprocessorNodeType;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, anychar};
use nom::combinator::{map, recognize, verify};
use nom::error::context;
use nom::multi::{many0, separated_list0};
use nom::sequence::{delimited, pair, tuple};

// /// Gathers multiple errors and contexts together
// pub type PpParserError<'a> = nom::error::VerboseError<&'a str>;
//
// /// Generic return value from a Nom parser which takes &str and returns `Out`
// pub type PpParserResult<'a, Out> = nom::IResult<&'a str, Out, PpParserError<'a>>;
//
// /// Return value from a Nom parser which takes &str and returns `Arc<PpAst>`
// pub type ParserResult<AstNode><'a> = PpParserResult<'a, Arc<PpAst>>;
//
// /// Return value from a Nom parser which takes &str and returns `Vec<Arc<PpAst>>`
// pub type VecPpAstParserResult<'a> = PpParserResult<'a, Vec<Arc<PpAst>>>;
//
// /// Return value from a Nom parser which takes &str and returns `String`
// pub type PpStringParserResult<'a> = PpParserResult<'a, String>;
//
// /// Return value from a Nom parser which takes &str and returns `&str`
// pub type StrSliceParserResult<'a> = PpParserResult<'a, &'a str>;
//
// /// Return value from a Nom parser which takes &str and returns `()`
// pub type VoidParserResult<'a> = PpParserResult<'a, ()>;

/// Groups code for parsing preprocessor directives
pub struct PreprocessorParser {}

impl PreprocessorParser {
  // /// Parse a `Var1, Var2, ...` into a list
  // fn parse_comma_sep_varnames(input: ParserInput) -> PpParserResult<Vec<String>> {
  //   separated_list0(comma, parse_varname)(input)
  // }

  /// Parse a `Macroident1, Macroident2, ...` into a list
  pub(crate) fn comma_sep_macro_idents(input: ParserInput) -> ParserResult<Vec<String>> {
    map(separated_list0(comma_tag, Self::macro_ident), |idents| {
      idents.into_iter().map(|i| i.to_string()).collect()
    })(input)
  }

  pub(crate) fn parenthesis_dot_newline(input: ParserInput) -> ParserResult<ParserInput> {
    recognize(tuple((par_close_tag, period_tag, newline_or_eof)))(input)
  }

  /// Parse an identifier, starting with a letter and also can be containing numbers and underscoress
  pub(crate) fn macro_ident(input: ParserInput) -> ParserResult<ParserInput> {
    recognize(pair(
      verify(anychar, |c: &char| c.is_alphabetic() || *c == '_'),
      many0(alt((alphanumeric1, tag("_".into())))),
    ))(input)
  }

  /// Parse a `-include(STRING)`
  fn include_directive(input: ParserInput) -> ParserResult<AstNode> {
    map(
      delimited(
        match_dash_tag("include".into()),
        delimited(par_open_tag, ws_before(parse_doublequot_string), par_close_tag),
        period_newline_tag,
      ),
      |t| PreprocessorNodeType::new_include(input.loc(), t),
    )(input.clone())
  }

  /// Parse a `-include_lib(STRING)`
  fn include_lib_directive(input: ParserInput) -> ParserResult<AstNode> {
    map(
      delimited(
        match_dash_tag("include_lib".into()),
        delimited(par_open_tag, ws_before(parse_doublequot_string), par_close_tag),
        period_newline_tag,
      ),
      |t| PreprocessorNodeType::new_include_lib(input.loc(), t),
    )(input.clone())
  }

  /// Parse one of supported preprocessor directives
  pub fn parse_preproc_directive(input: ParserInput) -> ParserResult<AstNode> {
    ws_before_mut(alt((
      // -define is special, it needs closing ).\n to consume the content
      context("'-define' directive", Self::define_directive),
      context("'-undef' directive", Self::undef_directive),
      // temporary nodes used by parse_if_block
      context("'-endif' directive", Self::endif_temporary_directive),
      context("'-elif' directive", Self::elif_temporary_directive),
      context("'-else' directive", Self::else_temporary_directive),
      context("'-ifdef' directive", Self::ifdef_temporary_directive),
      context("'-ifndef' directive", Self::ifndef_temporary_directive),
      context("'-if' directive", Self::if_block), // if must go after longer words ifdef and ifndef
      // Self::parse_error,
      // Self::parse_warning,
      context("'-include_lib' directive", Self::include_lib_directive),
      context("'-include' directive", Self::include_directive),
    )))(input)
  }

  // /// Parse full lines till a line which looks like a preprocessor directive is found
  // fn consume_one_line_of_text(input: ParserInput) -> ParserResult<AstNode> {
  //   map(
  //     verify(
  //       ws(nom::bytes::complete::take_till(|c| c == '\n' || c == '\r')),
  //       |text: &str| !text.is_empty(), //&& !text.starts_with('-'),
  //     ),
  //     |t| PreprocessorNodeType::new_text(SourceLoc::from_input(input), t),
  //   )(input)
  // }

  // /// Parses either a preprocessor directive or block, or consumes one line of text
  // pub(crate) fn parse_fragment(input: ParserInput) -> ParserResult<AstNode> {
  //   alt((
  //     Self::parse_preproc_directive,
  //     // Self::consume_one_line_of_text,
  //     // A final comment in file is not visible to consume_text
  //     map(parse_line_comment, |_| {
  //       PreprocessorNodeType::new_text(SourceLoc::from_input(input), "")
  //     }),
  //   ))(input)
  // }

  // /// Split input into AST nodes for preprocessor directives and any irrelevant text in between
  // pub fn parse_fragments_collection(input: ParserInput) -> VecAstParserResult {
  //   // Followed by 1 or more directive or another text fragment
  //   many1(Self::parse_fragment)(input)
  // }

  // /// Parses file contents into mix of preprocessor directives and text fragments.
  // /// Comments are eliminated.
  // pub fn module(input: ParserInput) -> ParserResult<AstNode> {
  //   map(Self::parse_fragments_collection, |fragments| {
  //     PreprocessorNodeType::new_file(SourceLoc::from_input(input), fragments)
  //   })(input)
  // }
}