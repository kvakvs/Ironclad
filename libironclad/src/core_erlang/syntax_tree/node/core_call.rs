//! A function call to a local fn or an exported fn from another module
#![cfg(coreast)]
use std::sync::Arc;

use crate::mfarity::MFArity;
use crate::core_erlang::syntax_tree::core_ast::CoreAst;
use crate::source_loc::SourceLoc;

/// Contains a function call on a MFArity
#[derive(Debug)]
#[cfg(coreast)]
pub struct Call {
  /// Source file pointer
  location: SourceLoc,
  /// Must exist
  pub target: MFArity,
  /// Must match arity
  pub args: Vec<Arc<CoreAst>>,
}