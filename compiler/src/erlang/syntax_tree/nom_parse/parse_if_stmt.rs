//! Parse code for `if COND -> EXPR; ... end`

use crate::erlang::syntax_tree::nom_parse::{AstParserResult, ErlParser, ErlParserError};
use nom::{bytes, bytes::complete::{tag}, character::complete::{char}, error::{context},
          combinator, combinator::{cut}, sequence, multi};
use crate::erlang::syntax_tree::erl_ast::ErlAst;
use crate::erlang::syntax_tree::node::erl_if_clause::ErlIfClause;
use crate::source_loc::SourceLoc;

impl ErlParser {
  /// Parses `if COND -> EXPR; ... end`
  pub fn parse_if_statement(input: &str) -> AstParserResult {
    combinator::map(
      sequence::delimited(
        tag("if"),
        multi::separated_list1(
          char(';'),
          context("if statement clause", cut(Self::parse_if_clause))
        ),
        tag("end"),
      ),
      |clauses| ErlAst::new_if_statement(SourceLoc::None, clauses),
    )(input)
  }

  /// Parses a `Condition -> ...` branch of `if COND -> EXPR; ... end` statement
  pub fn parse_if_clause(input: &str) -> nom::IResult<&str, ErlIfClause, ErlParserError> {
    combinator::map(
      sequence::tuple((
        Self::parse_expr,

        // The body after ->
        sequence::preceded(
          Self::ws_before(bytes::complete::tag("->")),
          Self::parse_expr,
        )
      )),
      |(cond, body)| ErlIfClause::new(cond, body),
    )(input)
  }
}