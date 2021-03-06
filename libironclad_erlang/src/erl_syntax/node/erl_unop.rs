//! Defines structs for AST nodes representing binary operators (A + B) and unary (+A)
use crate::erl_syntax::erl_ast::node_impl::AstNodeImpl;
use crate::erl_syntax::erl_ast::node_impl::AstNodeType::UnaryOp;
use crate::erl_syntax::erl_ast::AstNode;
use crate::erl_syntax::erl_op::ErlUnaryOp;
use crate::erl_syntax::literal_bool::LiteralBool;
use crate::source_loc::SourceLoc;

/// Unary operator is right-associative operation such as `not A` or `+A`
#[derive(Debug)]
pub struct ErlUnaryOperatorExpr {
  /// The operand
  pub expr: AstNode,
  /// The operator
  pub operator: ErlUnaryOp,
}

impl ErlUnaryOperatorExpr {
  /// Create an unary operator and wrap it with ErlAst::UnaryOp
  pub(crate) fn new_ast(loc: SourceLoc, operator: ErlUnaryOp, expr: AstNode) -> AstNode {
    let unop_node = UnaryOp { unop_expr: ErlUnaryOperatorExpr { expr, operator } };
    AstNodeImpl::construct_with_location(loc, unop_node)
  }

  /// Walk the literal expression and try to find whether it is true, false or neither
  #[allow(dead_code)]
  pub(crate) fn walk_boolean_litexpr(&self) -> LiteralBool {
    match self.operator {
      ErlUnaryOp::Not => self.expr.walk_boolean_litexpr().negate(),
      _ => LiteralBool::NotABoolean,
    }
  }
}
