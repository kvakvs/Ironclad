use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use crate::project::source_file::SourceFile;

/// Defines a root contents of a module containing generic AST elements.
/// It could be preprocessor Ast tree, or Erlang Ast tree or something else.
#[derive(Debug)]
pub struct AstTree<TNode> {
  pub source: Arc<SourceFile>,
  pub nodes: Vec<TNode>,
}

impl<TNode> AstTree<TNode> {
  pub fn new(source_file: Arc<SourceFile>, forms: Vec<TNode>) -> Self {
    Self {
      source: source_file,
      nodes: forms,
    }
  }
}

pub struct AstCache<TNode> {
  pub items: HashMap<PathBuf, Arc<AstTree<TNode>>>,
}

impl<TNode> AstCache<TNode> {
  pub fn new_empty() -> Self {
    Self { items: Default::default() }
  }
}
