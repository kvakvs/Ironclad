//! Record syntax parser support

use crate::erl_syntax::node::erl_record::RecordField;
use crate::erl_syntax::parsers::defs::ParserResult;
use crate::erl_syntax::parsers::misc::{dash_atom, period_eol_eof, tok_atom};
use crate::erl_syntax::parsers::misc_tok::*;
use crate::erl_syntax::parsers::parse_expr::parse_expr;
use crate::erl_syntax::parsers::parse_type::parse_type;
use crate::erl_syntax::parsers::parser_input::ParserInput;
use crate::erl_syntax::preprocessor::pp_node::pp_impl::PreprocessorNodeImpl;
use crate::erl_syntax::preprocessor::pp_node::PreprocessorNode;
use crate::source_loc::SourceLoc;
use nom::combinator::{cut, map, opt};
use nom::error::context;
use nom::multi::separated_list0;
use nom::sequence::{delimited, preceded, separated_pair, tuple};

/// Parses one field from the field list of `-record(atom(), { <FIELDS> } ).`.
/// The field parser has a structure: `ATOM ( = EXPR ) ( :: TYPE )`
fn record_definition_one_field(input: ParserInput) -> ParserResult<RecordField> {
  map(
    tuple((
      tok_atom,
      opt(preceded(
        tok_equal_symbol,
        context("default value for a field", cut(parse_expr)),
      )),
      opt(preceded(
        tok_double_colon,
        context("type ascription for a field", cut(parse_type)),
      )),
    )),
    |(field_tag, opt_initializer, opt_type)| RecordField {
      field_tag,
      initializer: opt_initializer,
      type_ascription: opt_type,
    },
  )(input)
}

/// Parses inner fields list of `-record(atom, { <FIELDS> } ).`
fn record_definition_fields(input: ParserInput) -> ParserResult<Vec<RecordField>> {
  delimited(
    tok_curly_open,
    separated_list0(tok_comma, context("record definition field", record_definition_one_field)),
    tok_curly_close,
  )(input)
}

/// Parses inner contents of `-record( <INNER> ).`
fn record_definition_inner(input: ParserInput) -> ParserResult<PreprocessorNode> {
  map(
    separated_pair(tok_atom, tok_comma, record_definition_fields),
    |(atom, fields)| {
      PreprocessorNodeImpl::new_record_definition(SourceLoc::new(&input), atom, fields)
    },
  )(input.clone())
}

/// Parses a `-record(atom(), {field :: type()... }).` attribute.
pub fn parse_record_def(input: ParserInput) -> ParserResult<PreprocessorNode> {
  delimited(
    |i1| dash_atom(i1, "record"),
    delimited(
      tok_par_open,
      context("record definition in a -record() attribute", cut(record_definition_inner)),
      tok_par_close,
    ),
    period_eol_eof,
  )(input)
}
