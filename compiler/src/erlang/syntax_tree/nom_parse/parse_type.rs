//! Contains parsers for function typespecs and type syntax.

use std::sync::Arc;
use nom::{combinator, sequence, multi, character, bytes::complete::{tag}, branch,
          combinator::{cut},
          error::{context}};

use crate::erlang::syntax_tree::erl_ast::ErlAst;
use crate::erlang::syntax_tree::nom_parse::{ErlParser, ErlParserError};
use crate::erlang::syntax_tree::nom_parse::parse_atom::AtomParser;
use crate::mfarity::MFArity;
use crate::source_loc::SourceLoc;
use crate::typing::erl_type::ErlType;
use crate::typing::fn_clause_type::FnClauseType;
use crate::typing::typevar::Typevar;

impl ErlParser {
  /// Given function spec module attribute `-spec name(args...) -> ...` parse into an AST node
  pub fn parse_fn_spec(input: &str) -> nom::IResult<&str, Arc<ErlAst>, ErlParserError> {
    combinator::map(
      sequence::tuple((
        Self::ws_before(character::complete::char('-')),
        Self::ws_before(tag("spec")),
        Self::ws_before(AtomParser::parse_atom),
        multi::separated_list1(
          Self::ws_before(character::complete::char(';')),
          context("function clause spec",
                  cut(Self::ws_before(Self::parse_fn_spec_fnclause))),
        ),
        Self::attr_terminator,
      )),
      |(_minus, _spec, name, clauses, _term)| {
        let arity = clauses[0].arity();
        assert!(clauses.iter().all(|c| c.arity() == arity),
                "All function clauses must have same arity in a typespec");
        let funarity = MFArity::new_local(&name, arity);
        let fntypespec = ErlType::new_fn_type(&clauses);
        let fnspec = ErlAst::FnSpec {
          location: SourceLoc::None,
          funarity,
          spec: fntypespec.into(),
        };
        fnspec.into()
      },
    )(input)
  }

  /// Parses a function clause args specs, return spec and optional `when`
  fn parse_fn_spec_fnclause(input: &str) -> nom::IResult<&str, FnClauseType, ErlParserError> {
    combinator::map(
      sequence::tuple((
        // Function clause name
        Self::ws_before_mut(combinator::opt(AtomParser::parse_atom)),

        // Args list (list of type variables with some types possibly)
        context("arguments list in a function clause spec",
                Self::parse_parenthesized_arg_spec_list),
        Self::ws_before(tag("->")),

        // Return type for fn clause
        context("return type in function clause spec",
                branch::alt((
                  Self::parse_typevar_with_opt_type,
                  Self::parse_type_as_typevar
                )),
        ),

        // Optional: when <comma separated list of typevariables given types>
        context("when expression for typespec",
                combinator::opt(Self::parse_when_expr_for_type)),
      )),
      |(_name, args, _arrow, ret_ty, when_expr)| {
        // TODO: Check name equals function name, for module level functions
        if when_expr.is_some() {
          let when_expr_val = when_expr.unwrap();
          FnClauseType::new(
            Typevar::merge_lists(&args, &when_expr_val),
            Typevar::substitute_var_from_when_clause(&ret_ty, &when_expr_val).clone(),
          )
        } else {
          FnClauseType::new(args, ret_ty.clone())
        }
      },
    )(input)
  }

  /// Parse part of typevar: `:: type()`, this is to be wrapped in `branch::opt()` by the caller
  fn parse_coloncolon_type(input: &str) -> nom::IResult<&str, Arc<ErlType>, ErlParserError> {
    let (input, _tag) = Self::ws_before(tag("::"))(input)?;
    context("::type()", Self::ws_before(Self::parse_type))(input)
  }

  /// Parse a capitalized type variable name with an optional `:: type()` part:
  /// `A :: type()` or `A`
  fn parse_typevar_with_opt_type(input: &str) -> nom::IResult<&str, Typevar, ErlParserError> {
    combinator::map(
      sequence::pair(
        Self::parse_typevar_name,
        combinator::opt(Self::parse_coloncolon_type),
      ),
      |(tv_name, maybe_type)| Typevar::new(Some(tv_name), maybe_type),
    )(input)
  }

  fn parse_type_as_typevar(input: &str) -> nom::IResult<&str, Typevar, ErlParserError> {
    combinator::map(
      Self::parse_type,
      |t| Typevar::from_erltype(&t),
    )(input)
  }

  // /// Parses a list of comma separated typevars (function arg specs)
  // fn parse_comma_sep_arg_specs(input: &str) -> nom::IResult<&str, Vec<Typevar>> {
  //   multi::separated_list0(
  //     Self::ws(character::complete::char(',')),
  //
  //     // Comma separated arguments spec can be typevars with optional `::type()`s or just types
  //     branch::alt((
  //       Self::parse_typevar_with_opt_type,
  //       combinator::map(Self::parse_type, |t| Typevar::from_erltype(&t)),
  //     )),
  //   )(input)
  // }

  /// Parses a list of comma separated typevars enclosed in (parentheses)
  pub fn parse_parenthesized_arg_spec_list(input: &str) -> nom::IResult<&str, Vec<Typevar>, ErlParserError> {
    let (input, _) = Self::ws_before(character::complete::char('('))(input)?;

    sequence::terminated(
      Self::parse_comma_sep_typeargs0,
      Self::ws_before(character::complete::char(')')),
    )(input)
  }

  /// Parse a `when` clause where unspecced typevars can be given types, like:
  /// `-spec fun(A) -> A when A :: atom().`
  pub fn parse_when_expr_for_type(input: &str) -> nom::IResult<&str, Vec<Typevar>, ErlParserError> {
    let (input, _) = Self::ws_before(tag("when"))(input)?;
    Self::parse_comma_sep_typeargs1(input)
  }

  /// Parse only capitalized type variable name
  fn parse_typevar_name(input: &str) -> nom::IResult<&str, String, ErlParserError> {
    Self::ws_before(Self::parse_ident_capitalized)(input)
  }

  fn alt_typevar_or_type(input: &str) -> nom::IResult<&str, Typevar, ErlParserError> {
    branch::alt((
      Self::parse_typevar_with_opt_type,
      combinator::map(
        Self::parse_type,
        |t| Typevar::from_erltype(&t),
      ),
      // combinator::map(Self::parse_typevar, |tvname| Typevar::new(Some(tvname), None)),
    ))(input)
  }

  fn parse_typearg(input: &str) -> nom::IResult<&str, Typevar, ErlParserError> {
    combinator::map(
      Self::ws_before(Self::parse_type),
      |t| Typevar::from_erltype(&t),
    )(input)
  }

  /// Parses a comma separated list of 0 or more type arguments.
  /// A parametrized type accepts other types or typevar names
  fn parse_comma_sep_typeargs0(input: &str) -> nom::IResult<&str, Vec<Typevar>, ErlParserError> {
    multi::separated_list0(
      Self::ws_before(character::complete::char(',')),
      context("parsing items of a typeargs0_list", Self::alt_typevar_or_type),
    )(input)
  }

  /// Parses a comma separated list of 1 or more type arguments.
  /// A parametrized type accepts other types or typevar names
  fn parse_comma_sep_typeargs1(input: &str) -> nom::IResult<&str, Vec<Typevar>, ErlParserError> {
    multi::separated_list1(
      Self::ws(character::complete::char(',')),
      context("parsing items of a typeargs1_list", Self::alt_typevar_or_type),
    )(input)
  }

  /// Parse a user defined type with `name()` and 0 or more typevar args.
  fn parse_user_defined_type(input: &str) -> nom::IResult<&str, Arc<ErlType>, ErlParserError> {
    combinator::map(
      sequence::tuple((
        Self::ws_before(Self::parse_ident),
        Self::ws_before(character::complete::char('(')),
        context("type arguments for a user-defined type",
                Self::parse_comma_sep_typeargs0),
        Self::ws_before(character::complete::char(')')),
      )),
      |(type_name, _open, elements, _close)| {
        ErlType::from_name(type_name, &elements).into()
      },
    )(input)
  }

  /// Parse a list of types, returns a temporary list-type
  fn parse_type_list(input: &str) -> nom::IResult<&str, Arc<ErlType>, ErlParserError> {
    let (input, _open_tag) = Self::ws_before(character::complete::char('['))(input)?;

    combinator::map(
      sequence::terminated(
        context("type arguments for a list() type",
                Self::parse_comma_sep_typeargs0),
        Self::ws_before(character::complete::char(']')),
      ),
      |elements| ErlType::TypevarList(elements).into(),
    )(input)
  }

  /// Parse a tuple of types, returns a temporary tuple-type
  fn parse_type_tuple(input: &str) -> nom::IResult<&str, Arc<ErlType>, ErlParserError> {
    let (input, _open_tag) = Self::ws_before(character::complete::char('{'))(input)?;

    combinator::map(
      sequence::terminated(
        context("type arguments for a tuple() type",
                Self::parse_comma_sep_typeargs0),
        Self::ws_before(character::complete::char('}')),
      ),
      |elements| ErlType::TypevarList(elements).into(),
    )(input)
  }

  /// Parse any simple Erlang type without union. To parse unions use `parse_type`.
  pub fn parse_nonunion_type(input: &str) -> nom::IResult<&str, Arc<ErlType>, ErlParserError> {
    branch::alt((
      Self::parse_type_list,
      Self::parse_type_tuple,
      Self::parse_user_defined_type,
    ))(input)
  }

  /// Parse any Erlang type, simple types like `atom()` with some `(args)` possibly, but could also be
  /// a structured type like union of multiple types `atom()|number()`, a list or a tuple of types, etc
  pub fn parse_type(input: &str) -> nom::IResult<&str, Arc<ErlType>, ErlParserError> {
    combinator::map(
      sequence::pair(
        Self::ws_before(Self::parse_nonunion_type),
        context("union type",
                multi::many0(
                  sequence::pair(
                    Self::ws_before(character::complete::char('|')),
                    context("union type continuation",
                            Self::ws_before(Self::parse_nonunion_type)),
                  )),
        ),
      ),
      |(t1, tail)| {
        let mut types: Vec<Arc<ErlType>> = tail.iter()
            .map(|(_, ty)| ty)
            .cloned()
            .collect();
        types.insert(0, t1);
        // Merge maybe-union t1 and maybe-union t2 into one
        ErlType::new_union(&types)
      },
    )(input)
  }

  /// Wraps parsed type into a type-AST-node
  pub fn parse_type_node(input: &str) -> nom::IResult<&str, Arc<ErlAst>, ErlParserError> {
    combinator::map(
      Self::parse_type,
      |t| ErlAst::Type { location: SourceLoc::None, ty: t }.into(),
    )(input)
  }
}