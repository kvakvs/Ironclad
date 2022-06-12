//! Parsing tools for `-if` family of directives

use crate::erl_syntax::erl_ast::node_impl::{AstNodeImpl, AstNodeType};
use crate::erl_syntax::erl_ast::AstNode;
use crate::erl_syntax::literal_bool::LiteralBool;
use crate::erl_syntax::parsers::defs::{ParserInput, ParserResult, VecAstParserResult};
use crate::erl_syntax::parsers::misc::{
  match_dash_tag, par_close_tag, par_open_tag, period_newline_tag, ws_before,
};
use crate::erl_syntax::parsers::parse_expr::parse_expr;
use crate::erl_syntax::parsers::parse_one_module_form;
use crate::erl_syntax::preprocessor::ast::PreprocessorNodeType;
use crate::erl_syntax::preprocessor::ast::PreprocessorNodeType::{_TemporaryElse, _TemporaryEndif};
use crate::erl_syntax::preprocessor::parsers::preprocessor::macro_ident;
use nom::combinator::{map, opt, recognize, verify};
use nom::error::context;
use nom::multi::many0;
use nom::sequence::{delimited, pair, preceded, terminated, tuple};

/// Parses multiple lines of any directives except `-endif.` or `-else.`
fn parse_fragments_till_else(input: ParserInput) -> VecAstParserResult {
  many0(verify(parse_one_module_form, |frag: &AstNode| {
    !frag.is_else() && !frag.is_elseif() && !frag.is_endif()
  }))(input)
}

/// Parses multiple lines of any directives except `-endif.`
fn parse_fragments_till_endif(input: ParserInput) -> VecAstParserResult {
  many0(verify(parse_one_module_form, |frag: &AstNode| !frag.is_endif()))(input)
}

/// Parse a `-if(EXPR).` `<LINES>` then optional `-else. <LINES> -endif.`
pub fn parse_if_block(input: ParserInput) -> ParserResult<AstNode> {
  map(
    terminated(
      tuple((
        if_condition,
        // Consume lines and directives until an `-else` or `-endif`
        context("condition true section of a -if()", parse_fragments_till_else),
        // Optional -else. <LINES> block
        context(
          "condition false section of a -if()",
          opt(preceded(else_temporary_directive, parse_fragments_till_endif)),
        ),
      )),
      // Ending with an endif
      endif_temporary_directive,
    ),
    |(if_cond, branch_true, branch_false)| {
      if if_cond {
        PreprocessorNodeType::new_group_node_temporary(branch_true)
      } else {
        PreprocessorNodeType::new_group_node_temporary(branch_false.unwrap_or_default())
      }
    },
  )(input.clone())
}

/// Parse a `-if(EXPR).\n` and return a temporary node
pub fn if_condition(input: ParserInput) -> ParserResult<bool> {
  map(
    delimited(
      match_dash_tag("if".into()),
      delimited(par_open_tag, ws_before(parse_expr), par_close_tag),
      period_newline_tag,
    ),
    // Builds a temporary If node with erl expression in it
    |expr| match expr.walk_boolean_litexpr() {
      LiteralBool::False => false,
      LiteralBool::True => true,
      LiteralBool::NotABoolean => panic!("Bool expression is expected here"),
    },
  )(input.clone())
}

/// Parse a `-elif(EXPR)` into a temporary AST node
pub(crate) fn elif_temporary_directive(input: ParserInput) -> ParserResult<AstNode> {
  map(
    delimited(
      match_dash_tag("elif".into()),
      delimited(par_open_tag, ws_before(parse_expr), par_close_tag),
      period_newline_tag,
    ),
    |t| PreprocessorNodeType::new_elif_temporary(input.loc(), t),
  )(input.clone())
}

fn ifdef_condition(input: ParserInput) -> ParserResult<bool> {
  map(
    delimited(
      match_dash_tag("ifdef".into()),
      delimited(par_open_tag, macro_ident, par_close_tag),
      period_newline_tag,
    ),
    |tag: String| input.parser_scope.is_defined(&tag),
  )(input.clone())
}

/// Parse a `-ifdef(MACRO_NAME)`
pub(crate) fn ifdef_directive(input: ParserInput) -> ParserResult<AstNode> {
  map(
    terminated(
      tuple((
        ifdef_condition,
        context("condition true section of an -ifdef()", parse_fragments_till_else),
        context(
          "condition false section of an -ifdef()",
          opt(preceded(else_temporary_directive, parse_fragments_till_endif)),
        ),
      )),
      endif_temporary_directive,
    ),
    |(cond_true, branch_true, branch_false)| {
      PreprocessorNodeType::new_group_node_temporary(if cond_true {
        branch_true
      } else {
        branch_false.unwrap_or_default()
      })
    },
  )(input.clone())
}

/// Parse a `-ifndef(MACRO_NAME)`
pub(crate) fn ifndef_temporary_directive(input: ParserInput) -> ParserResult<AstNode> {
  map(
    delimited(
      match_dash_tag("ifndef".into()),
      delimited(par_open_tag, macro_ident, par_close_tag),
      period_newline_tag,
    ),
    |t: String| PreprocessorNodeType::new_ifndef_temporary(input.loc(), t),
  )(input.clone())
}

/// Parse a `-else.`, return a temporary `Else` node, which will not go into final `PpAst`
pub(crate) fn else_temporary_directive(input: ParserInput) -> ParserResult<AstNode> {
  map(
    delimited(
      match_dash_tag("else".into()),
      opt(pair(par_open_tag, par_close_tag)),
      period_newline_tag,
    ),
    |_opt| PreprocessorNodeType::construct_with_location(input.loc(), _TemporaryElse),
  )(input.clone())
}

fn maybe_empty_parens(input: ParserInput) -> ParserResult<ParserInput> {
  recognize(opt(pair(par_open_tag, par_close_tag)))(input)
}

/// Parse a `-endif.`, return a temporary `Endif` node, which will not go into final `PpAst`
pub(crate) fn endif_temporary_directive(input: ParserInput) -> ParserResult<AstNode> {
  map(
    delimited(match_dash_tag("endif".into()), maybe_empty_parens, period_newline_tag),
    |_opt| PreprocessorNodeType::construct_with_location(input.loc(), _TemporaryEndif),
  )(input.clone())
}