//! Haskell-style `let x = Value in Expr` node
use crate::core_erlang::syntax_tree::core_ast::CoreAst;
use crate::core_erlang::syntax_tree::node::var::Var;
use crate::typing::typevar::TypeVar;

/// Represents Haskell-style `let x = Value in Expr` node, used in Core Erlang to create scopes
pub struct LetExpr {
  /// The variable name assigned in let..in
  pub var: Vec<Var>,

  /// Value (type is in it), must return a vector of as many values, as `Self::var` has variables.
  pub value: Box<CoreAst>,

  /// Let x=y in <body> (type is in it, and becomes type of Expr::Let)
  pub in_expr: Box<CoreAst>,
  /// The `let` node result type
  pub ret_ty: TypeVar,
}