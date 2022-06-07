//! Construction routines for Preprocessor AST nodes

use crate::erl_syntax::erl_ast::node_impl::{AstNodeImpl, ErlAstType};
use crate::erl_syntax::erl_ast::AstNode;
use crate::erl_syntax::preprocessor::ast::PreprocessorNodeType;
use crate::erl_syntax::preprocessor::ast::PreprocessorNodeType::{
  Define, IfBlock, Include, IncludeLib, IncludedFile, Undef, _TemporaryElseIf, _TemporaryIf,
  _TemporaryIfdef, _TemporaryIfndef,
};
use crate::source_loc::SourceLoc;
use std::path::{Path, PathBuf};

impl PreprocessorNodeType {
  /// Generic constructor no location
  #[inline]
  pub fn construct_without_location(node_type: PreprocessorNodeType) -> AstNode {
    AstNodeImpl::construct_with_location(SourceLoc::None, ErlAstType::Preprocessor(node_type))
  }

  /// Generic constructor + location
  #[inline]
  pub fn construct_with_location(loc: SourceLoc, node_type: PreprocessorNodeType) -> AstNode {
    AstNodeImpl::construct_with_location(loc, ErlAstType::Preprocessor(node_type))
  }

  /// Create new nested included file AST node
  pub fn new_included_file(location: SourceLoc, file: &Path, ast: AstNode) -> AstNode {
    Self::construct_with_location(location, IncludedFile { filename: PathBuf::from(file), ast })
  }

  /// Create new macro definition
  pub fn new_define(location: SourceLoc, name: String, args: Vec<String>, body: String) -> AstNode {
    Self::construct_with_location(location, Define { name, args, body })
  }

  /// Create new macro definition with name only
  pub fn new_define_name_only(location: SourceLoc, name: String) -> AstNode {
    Self::construct_with_location(
      location,
      Define {
        name,
        args: Vec::default(),
        body: String::default(),
      },
    )
  }

  // /// Create new text fragment
  // pub fn new_text(location: SourceLoc, text: &str) -> AstNode {
  //   if text.trim().is_empty() {
  //     Self::construct_with_location(location, PreprocessorNodeType::EmptyText)
  //   } else {
  //     Self::construct_with_location(location, PreprocessorNodeType::Text(String::new(text)))
  //   }
  // }

  /// Creates a new preprocessor IF node
  pub fn new_if(
    location: SourceLoc,
    expr: AstNode,
    cond_true: Vec<AstNode>,
    cond_false: Vec<AstNode>,
  ) -> AstNode {
    Self::construct_with_location(location, IfBlock { cond: expr, cond_true, cond_false })
  }

  /// Create a new `-if()` temporary node.
  pub fn new_if_temporary(location: SourceLoc, expr: AstNode) -> AstNode {
    Self::construct_with_location(location, _TemporaryIf(expr))
  }

  /// Create a new `-elif()` temporary node.
  pub fn new_elif_temporary(location: SourceLoc, expr: AstNode) -> AstNode {
    Self::construct_with_location(location, _TemporaryElseIf(expr))
  }

  /// Create a new `-ifdef()` temporary node.
  pub fn new_ifdef_temporary(location: SourceLoc, ident: String) -> AstNode {
    Self::construct_with_location(location, _TemporaryIfdef(ident))
  }

  /// Create a new `-ifndef()` temporary node.
  pub fn new_ifndef_temporary(location: SourceLoc, ident: String) -> AstNode {
    Self::construct_with_location(location, _TemporaryIfndef(ident))
  }

  /// Create a new UNDEF node
  pub fn new_undef(location: SourceLoc, ident: String) -> AstNode {
    Self::construct_with_location(location, Undef(ident))
  }

  /// Create a new INCLUDE node
  pub fn new_include(location: SourceLoc, p: String) -> AstNode {
    Self::construct_with_location(location, Include(p))
  }

  /// Create a new INCLUDE_LIB node
  pub fn new_include_lib(location: SourceLoc, p: String) -> AstNode {
    Self::construct_with_location(location, IncludeLib(p))
  }
}