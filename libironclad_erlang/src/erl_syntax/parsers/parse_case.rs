//! Parses `case of` and clause branches
use crate::erl_syntax::erl_ast::node_impl::AstNodeImpl;
use crate::erl_syntax::erl_ast::AstNode;
use crate::erl_syntax::node::erl_case_clause::ErlCaseClause;
use crate::erl_syntax::parsers::defs::ParserResult;
use crate::erl_syntax::parsers::misc_tok::*;
use crate::erl_syntax::parsers::parse_expr::{
  parse_comma_sep_exprs1, parse_expr, parse_guardexpr, parse_matchexpr,
};
use crate::erl_syntax::parsers::parser_input::ParserInput;
use crate::source_loc::SourceLoc;
use nom::combinator::{cut, map, opt};
use nom::error::context;
use nom::multi::separated_list1;
use nom::sequence::{pair, preceded, terminated, tuple};

/// Parses a `MATCH_EXPR when GUARD_EXPR -> EXPR` branch of a `case` or a `try of`
pub(crate) fn parse_case_clause(input: ParserInput) -> ParserResult<ErlCaseClause> {
  map(
    tuple((
      parse_matchexpr,
      opt(preceded(
        keyword_when,
        context("case clause guard expression", cut(parse_guardexpr)),
      )),
      // The body after ->
      preceded(tok_right_arrow, context("case clause body", cut(parse_comma_sep_exprs1))),
    )),
    |(pattern, maybe_when, body)| {
      ErlCaseClause::new(
        pattern,
        maybe_when,
        AstNodeImpl::new_comma_expr(SourceLoc::new(&input), body),
      )
    },
  )(input.clone())
}

/// Parses `case EXPR of MATCH -> EXPR; ... end`
pub(crate) fn parse_case_expression(input: ParserInput) -> ParserResult<AstNode> {
  let map_fn = |(expr, clauses): (AstNode, Vec<ErlCaseClause>)| -> AstNode {
    AstNodeImpl::new_case_statement(SourceLoc::new(&input), expr, clauses)
  };
  preceded(
    keyword_case,
    context(
      "case-of-end expression",
      cut(map(
        terminated(
          pair(
            terminated(context("case block expression", cut(parse_expr)), keyword_of),
            context(
              "case clauses list",
              separated_list1(tok_semicolon, context("case block clause", cut(parse_case_clause))),
            ),
          ),
          keyword_end,
        ),
        map_fn,
      )),
    ),
  )(input.clone())
}
