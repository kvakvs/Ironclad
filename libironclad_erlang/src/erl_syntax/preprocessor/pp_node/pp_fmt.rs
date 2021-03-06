//! Printing preprocessor AST nodes

use crate::erl_syntax::parsers::token_stream::token::format_tok_stream;
use crate::erl_syntax::preprocessor::pp_node::pp_impl::PreprocessorNodeImpl;
use crate::erl_syntax::preprocessor::pp_node::pp_type::PreprocessorNodeType;
use ::function_name::named;
use libironclad_util::pretty::Pretty;

impl std::fmt::Display for PreprocessorNodeImpl {
  /// Format Preprocessor AST as a string
  #[named]
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self.content {
      PreprocessorNodeType::Include(p) => writeln!(f, "-include(\"{}\").", p),
      PreprocessorNodeType::IncludeLib(p) => write!(f, "-include_lib(\"{}\").", p),
      // PreprocessorNodeType::IncludedFile { tokens, filename } => {
      //   writeln!(f, "%% included from: {}", filename.to_string_lossy())?;
      //   format_tok_stream(tokens, 100).fmt(f)
      // }
      PreprocessorNodeType::Define { name, args, body } => {
        write!(f, "-define({}", name)?;

        if !args.is_empty() {
          Pretty::display_paren_list(args.iter(), f)?;
          write!(f, ").")?;
        }
        if !body.is_empty() {
          write!(f, ", {}).", format_tok_stream(body, 100))?;
        }
        Ok(())
      }

      PreprocessorNodeType::Error(t) => {
        write!(f, "-error(")?;
        Pretty::doublequot_string(f, t)?;
        write!(f, ").")
      }
      PreprocessorNodeType::Warning(t) => {
        write!(f, "-warning(")?;
        Pretty::doublequot_string(f, t)?;
        write!(f, ").")
      }
      PreprocessorNodeType::Attr { tag, term, .. } => {
        if let Some(t) = term {
          writeln!(f, "-{}({}).", tag, t)
        } else {
          writeln!(f, "-{}.", tag)
        }
      }
      PreprocessorNodeType::Export { fun_arities, .. } => {
        write!(f, "-export(")?;
        Pretty::display_square_list(fun_arities.iter(), f)?;
        writeln!(f, ").")
      }
      PreprocessorNodeType::ExportType { type_arities, .. } => {
        write!(f, "-export_type(")?;
        Pretty::display_square_list(type_arities.iter(), f)?;
        writeln!(f, ").")
      }
      PreprocessorNodeType::Import { module, fun_arities, .. } => {
        write!(f, "-import({}, ", module)?;
        Pretty::display_square_list(fun_arities.iter(), f)?;
        writeln!(f, ").")
      }
      PreprocessorNodeType::NewType { name, vars, ty, .. } => {
        write!(f, "-type {}", name)?;
        Pretty::display_paren_list(vars.iter(), f)?;
        write!(f, " :: {}", ty)?;
        writeln!(f, ".")
      }
      PreprocessorNodeType::NewRecord { tag, fields } => {
        write!(f, "-record({}, {{", tag)?;
        Pretty::display_comma_separated(fields.iter(), f)?;
        write!(f, "}}")
      }
      PreprocessorNodeType::FnSpec { funarity, spec, .. } => {
        write!(f, "-spec {}", funarity.name)?;
        Pretty::display_semicolon_separated(spec.as_fn_type().clauses().iter(), f)?;
        write!(f, ".")
      }
      PreprocessorNodeType::ModuleName { name, .. } => write!(f, "-module({}).", name),
      PreprocessorNodeType::Else => write!(f, "-else."),
      PreprocessorNodeType::Endif => write!(f, "-endif."),
      PreprocessorNodeType::If { cond } => write!(f, "-if({}).", cond),
      PreprocessorNodeType::ElseIf { cond } => write!(f, "-elseif({}).", cond),
      PreprocessorNodeType::Ifdef { macro_name, .. } => write!(f, "-ifdef({}).", macro_name),
      PreprocessorNodeType::Ifndef { macro_name, .. } => write!(f, "-ifndef({}).", macro_name),
      PreprocessorNodeType::Undef(name) => write!(f, "-undef({}).", name),
      #[allow(unreachable_patterns)]
      _ => unreachable!("{}(): can't process {:?}", function_name!(), self),
    }
  }
}
