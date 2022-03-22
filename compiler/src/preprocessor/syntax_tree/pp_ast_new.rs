//! Construction routines for PpAst

use std::path::{Path, PathBuf};
use std::sync::Arc;
use crate::preprocessor::syntax_tree::pp_ast::PpAst;

impl PpAst {
  /// Create new file tree
  pub fn new_file(fragments: Vec<Arc<PpAst>>) -> Arc<Self> {
    PpAst::File(fragments).into()
  }

  /// Create new nested included file AST node
  pub fn new_included_file(file: &Path, ast: Arc<PpAst>) -> Arc<Self> {
    PpAst::IncludedFile {
      filename: PathBuf::from(file),
      nested: ast,
    }.into()
  }
}