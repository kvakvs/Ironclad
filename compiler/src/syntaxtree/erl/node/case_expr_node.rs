//! Defines CaseExpr struct for `case X of` AST node
use crate::syntaxtree::erl::erl_ast::ErlAst;
use crate::typing::erl_type::ErlType;

/// `Case X of ... end` expression AST node
// #[derive(PartialEq)]
pub struct CaseExprNode {
  /// A union type of all case clauses, also is the return type of the case expression
  pub ret: ErlType,
  /// Argument of the `case X of`
  pub arg: Box<ErlAst>,
  /// All case clauses in order
  pub clauses: Vec<ErlAst>, // TODO: turn into Vec<CClause>
}
