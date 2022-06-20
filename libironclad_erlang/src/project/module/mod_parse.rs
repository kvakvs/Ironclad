//! Parsing impl for `ErlModule`

use crate::erl_syntax::erl_ast::AstNode;
use crate::erl_syntax::parsers::defs::ParserResult;
use crate::erl_syntax::parsers::misc::panicking_parser_error_reporter;
use crate::erl_syntax::parsers::parse_expr::parse_expr;
use crate::erl_syntax::parsers::parse_fn::parse_fndef;
use crate::erl_syntax::parsers::parse_module;
use crate::erl_syntax::parsers::parse_type::ErlTypeParser;
use crate::erl_syntax::parsers::parser_input::ParserInput;
use crate::erl_syntax::token_stream::tokenizer::tokenize_source;
use crate::error::ic_error::IcResult;
use crate::project::compiler_opts::CompilerOpts;
use crate::project::module::mod_impl::{ErlModule, ErlModuleImpl};
use crate::project::ErlProject;
use crate::source_file::SourceFile;
use nom::Finish;
use std::sync::RwLock;

impl ErlModuleImpl {
  /// Generic parse helper for any Nom entry point.
  /// Input comes as string in the `SourceFile`, the input is tokenized and then parsed.
  pub fn parse_helper<T>(
    project: &ErlProject,
    src_file: SourceFile,
    parse_fn: T,
    compiler_options: Option<CompilerOpts>,
  ) -> IcResult<ErlModule>
  where
    T: Fn(ParserInput) -> ParserResult<AstNode>,
  {
    let mut module_impl = ErlModuleImpl {
      source_file: src_file.clone(),
      ..ErlModuleImpl::default()
    };
    if let Some(o) = compiler_options {
      module_impl.compiler_options = o;
    }
    let module: ErlModule = RwLock::new(module_impl).into();

    //----------------------
    // Stage 1 tokenize the input
    //----------------------
    let tok_stream1 = ErlModuleImpl::tokenize_helper(project, src_file.clone(), tokenize_source)?;

    //----------------------
    // Stage 2 preprocessor: handle ifdefs, defines, includes etc
    // tokenize includes and paste in the token stream too
    //----------------------
    let tok_stream2 = ErlModuleImpl::preprocess(&module, &tok_stream1)?;

    //----------------------
    // Stage 3 real parsing begins: tokens to AST
    //----------------------
    let (tail, forms) = {
      let tokens_input = ParserInput::new(&src_file, &tok_stream2);
      panicking_parser_error_reporter(
        src_file.text.as_str(),
        tokens_input.clone(),
        parse_fn(tokens_input.clone()).finish(),
      )
    };

    assert!(
      tail.is_empty(),
      "Not all input was consumed by parse.\n\tTail: «{:?}»\n\tForms: {}",
      tail,
      forms
    );

    // TODO: This assignment below should be happening earlier before parse, as parse can refer to the SourceFile
    if let Ok(mut w_module) = module.write() {
      w_module.ast = forms;
    } else {
      panic!("Can't lock module for updating AST field")
    }

    // Scan AST and find FnDef nodes, update functions knowledge
    // Scope::update_from_ast(&module.scope, &module.ast);

    Ok(module)
  }

  /// Parses code fragment starting with "-module(...)." and containing some function definitions
  /// and the usual module stuff.
  pub fn from_module_source(
    project: &ErlProject,
    source_file: &SourceFile,
    compiler_options: Option<CompilerOpts>,
  ) -> IcResult<ErlModule> {
    Self::parse_helper(project, source_file.clone(), parse_module, compiler_options)
  }

  /// Creates a module, where its AST comes from an expression
  pub fn from_expr_source(
    project: &ErlProject,
    source_file: &SourceFile,
    compiler_options: Option<CompilerOpts>,
  ) -> IcResult<ErlModule> {
    Self::parse_helper(project, source_file.clone(), parse_expr, compiler_options)
  }

  /// Creates a module, where its AST comes from a function
  pub fn from_fun_source(
    project: &ErlProject,
    source_file: &SourceFile,
    compiler_options: Option<CompilerOpts>,
  ) -> IcResult<ErlModule> {
    Self::parse_helper(project, source_file.clone(), parse_fndef, compiler_options)
  }

  // /// Creates a 'module', where its AST comes from a typespec source `-spec myfun(...) -> ...`
  // pub fn from_fun_spec_source(
  //   project: &ErlProject,
  //   source_file: &SourceFile,
  //   compiler_options: Option<CompilerOpts>,
  // ) -> IcResult<ErlModule> {
  //   Self::parse_helper(project, source_file.clone(), ErlTypeParser::fn_spec_attr, compiler_options)
  // }

  /// Creates a 'module', where its AST comes from a type `integer() | 42`
  pub fn from_type_source(
    project: &ErlProject,
    source_file: &SourceFile,
    compiler_options: Option<CompilerOpts>,
  ) -> IcResult<ErlModule> {
    Self::parse_helper(
      project,
      source_file.clone(),
      ErlTypeParser::parse_type_node,
      compiler_options,
    )
  }
}