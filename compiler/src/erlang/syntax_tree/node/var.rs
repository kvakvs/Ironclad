//! Defines struct for a variable AST node
use crate::typing::typevar::TypeVar;

/// AST node for a variable
#[derive(PartialEq)]
pub struct Var {
  /// Variable name
  pub name: String,
  /// Variable type for inference
  pub ty: TypeVar,
}

impl Var {
  /// Creates a new variable node
  pub fn new(name: &str) -> Self {
    Var {
      name: name.to_string(),
      ty: TypeVar::new(),
    }
  }
}