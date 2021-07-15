use crate::syntaxtree::ast_cache::{AstCache, AstTree};
use crate::syntaxtree::erl::literal::ErlLiteral;
use crate::typing::typevar::TypeVar;
use crate::typing::erl_type::ErlType;
use crate::syntaxtree::erl::erl_op::{ErlBinaryOp, ErlUnaryOp};
use std::borrow::Borrow;

#[derive(Debug, PartialEq)]
pub enum ErlAst {
  /// Forms list, root of a module
  Forms(Vec<ErlAst>),

  /// Generic module attribute -"string"(value, ...).
  ModuleAttr { name: String, args: Vec<String> },

  // Lit { value: ErlLiteral, ty: ErlType },
  // Variable { name: String, tv: TypeVar }, // variable is part of ErlExpr enum

  // /// Any expression, where its type initially is Any
  // Expr { expr: ErlExpr, ty: ErlType },

  /// Defines a new function.
  /// A function has clauses.
  /// Each clause has same quantity of args (some AST nodes), bindable expressions,
  /// and a return type, initially Any
  NewFunction {
    name: String,
    // Each clause is ErlExpr, and union of clause types will be function return type
    ret: ErlType,
    clauses: Vec<ErlAst>,
  },

  FClause {
    args: Vec<ErlAst>,
    arg_types: Vec<TypeVar>,
    body: Box<ErlAst>,
  },

  CClause {
    /// A match expression, matched vs. case arg
    cond: Box<ErlAst>,
    /// Must resolve to bool, or an exception
    guard: Box<ErlAst>,
    body: Box<ErlAst>,
  },

  /// A named variable
  Var {
    name: String,
    ty: ErlType,
  },

  /// Apply arguments to expression
  App {
    /// Target, to be called, expected to have function or lambda type
    expr: Box<ErlAst>,
    /// Arguments. Their  inferred types are stored inside.
    args: Vec<ErlAst>,
    /// Return inferred type.
    ty: ErlType,
  },

  // /// A lambda definition or a function
  // Function { args: Vec<ErlExpr>, expr: Box<ErlExpr> },

  /// A haskell-style new variable introducing a new scope below it:
  /// let x = expr1 in expr2
  Let {
    var: String,
    /// Type which we believe is Var
    var_ty: ErlType,
    /// Value (type is in it)
    value: Box<ErlAst>,
    /// Let x=y in <body> (type is in it, and becomes type of Expr::Let)
    in_expr: Box<ErlAst>,
  },

  // // TODO: Remove If because can be replaced with Case
  // If {
  //   cond: Box<ErlExpr>,
  //   on_true: Box<ErlExpr>,
  //   on_false: Box<ErlExpr>,
  // },
  Case {
    /// A union type of all case clauses
    ty: ErlType,
    arg: Box<ErlAst>,
    clauses: Vec<ErlAst>,
  },

  /// A literal value, constant. Type is known via literal.get_type()
  Lit(ErlLiteral),

  BinaryOp { left: Box<ErlAst>, right: Box<ErlAst>, op: ErlBinaryOp, ty: ErlType },
  UnaryOp { expr: Box<ErlAst>, op: ErlUnaryOp },
}

impl ErlAst {
  pub fn get_type(&self) -> ErlType {
    match self {
      ErlAst::Forms(_) => ErlType::Any,
      ErlAst::ModuleAttr { .. } => ErlType::Any,
      ErlAst::NewFunction { ret, .. } => ret.clone(),
      ErlAst::FClause { body, .. } => body.get_type(),
      ErlAst::CClause { body, .. } => body.get_type(),
      ErlAst::Var { ty, .. } => ty.clone(),
      ErlAst::App { ty, .. } => ty.clone(),
      ErlAst::Let { in_expr, .. } => in_expr.get_type(),
      ErlAst::Case { ty, .. } => ty.clone(),
      ErlAst::Lit(l) => l.get_type().clone(),
      ErlAst::BinaryOp { op,.. } => op.get_result_type(),
      ErlAst::UnaryOp { expr,.. } => expr.get_type(), // same type as expr bool or num
    }
  }

  /// Create a new function clause
  pub fn new_fclause(args: Vec<ErlAst>, expr: ErlAst) -> Self {
    let arg_types = args.iter().map(|_a| TypeVar::new()).collect();
    Self::FClause {
      args,
      arg_types,
      body: Box::from(expr),
    }
  }

  /// Build a vec of references to children
  pub fn get_children(&self) -> Option<Vec<&ErlAst>> {
    match self {
      ErlAst::Forms(f) => Some(f.iter().collect()),
      ErlAst::ModuleAttr { .. } => None,
      ErlAst::Lit { .. } => None,
      ErlAst::NewFunction { clauses, .. } => Some(clauses.iter().collect()),
      ErlAst::FClause { args, arg_types, body } => {
        // Descend into args, and the body
        let mut args_refs: Vec<&ErlAst> = args.iter().collect();
        args_refs.push(&body);
        Some(args_refs)
      }
      ErlAst::Var { .. } => None,
      ErlAst::App { expr, args, .. } => {
        let mut r = vec![expr.borrow()];
        args.iter().for_each(|a| r.push(a));
        Some(r)
      }
      ErlAst::Let { value, in_expr, .. } => {
        Some(vec![&value, &in_expr])
      }
      ErlAst::Case { arg, clauses, .. } => {
        let mut r = vec![arg.borrow()];
        clauses.iter().for_each(|a| r.push(a));
        Some(r)
      }
      ErlAst::CClause { cond, guard, body } => {
        Some(vec![cond.borrow(), guard.borrow(), body.borrow()])
      }
      ErlAst::BinaryOp { left, right, .. } => {
        Some(vec![left.borrow(), right.borrow()])
      }
      ErlAst::UnaryOp { expr, .. } => { Some(vec![expr.borrow()]) }
    }
  }

  // pub fn has_children(&self) -> bool {
  //   match self {
  //     // Descend into module contents
  //     ErlAst::Forms(_) => unreachable!("Do not call on module root ErlAst::Forms"),
  //     ErlAst::ModuleAttr { .. } => false,
  //     ErlAst::Lit { .. } => false, // TODO: nested literals?
  //     // Descend into args expressions, and into body
  //     ErlAst::NewFunction { .. } => true,
  //     ErlAst::Var { .. } => false,
  //     // Descend into app expression and args expressions
  //     ErlAst::App { .. } => true,
  //     // Descend into variable value, and in-body
  //     ErlAst::Let { .. } => true,
  //     // Descend into the condition, and clauses
  //     ErlAst::Case { .. } => true,
  //     // Descend into the args
  //     ErlAst::BinaryOp { .. } => true,
  //     // Descend into the arg
  //     ErlAst::UnaryOp { .. } => true,
  //   }
  // }

  pub fn new_fun(name: &str, clauses: Vec<ErlAst>) -> Self {
    ErlAst::NewFunction {
      name: name.to_string(),
      clauses,
      ret: ErlType::new_typevar(),
    }
  }

  pub fn new_var(name: &str) -> ErlAst {
    ErlAst::Var {
      name: name.to_string(),
      ty: ErlType::new_typevar(),
    }
  }
}

/// A tree of Erlang nodes with attached file name, and root element removed
pub(crate) type ErlAstTree = AstTree<ErlAst>;

/// A cache of trees of Erlang nodes, keyed by filename or module name
pub(crate) type ErlAstCache = AstCache<ErlAst>;
