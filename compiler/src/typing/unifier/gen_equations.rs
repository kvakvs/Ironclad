//! Type inference wiring.
//! Contains code for generating type equations from AST in `Unifier` impl
use std::ops::Deref;
use function_name::named;

use crate::typing::unifier::Unifier;
use crate::typing::unifier::equation::TypeEquation;
use crate::typing::erl_type::ErlType;
use crate::erl_error::ErlResult;
use crate::core_erlang::syntax_tree::core_ast::CoreAst;
use crate::core_erlang::syntax_tree::node::fn_def::FnDef;
use crate::core_erlang::syntax_tree::node::apply::Apply;
use crate::project::module::Module;
use std::sync::Arc;
use crate::core_erlang::syntax_tree::node::prim_op::PrimOp;
use crate::core_erlang::syntax_tree::node::case::Case;
use crate::core_erlang::syntax_tree::node::expression::BinaryOperatorExpr;

impl Unifier {
  /// Add a type equation, shortcut
  fn equation(eq: &mut Vec<TypeEquation>, ast: &Arc<CoreAst>,
              type_deduced: &Arc<ErlType>,
              type_expected: &Arc<ErlType>) {
    eq.push(
      TypeEquation::new(Arc::downgrade(ast),
                        type_deduced.clone(),
                        type_expected.clone(),
                        format!("{}", ast))
    )
  }

  /// Add a type equation, shortcut, with annotation
  fn equation_anno(eq: &mut Vec<TypeEquation>, ast: &Arc<CoreAst>,
                   type_deduced: &Arc<ErlType>,
                   type_expected: &Arc<ErlType>,
                   anno: String) {
    eq.push(
      TypeEquation::new(Arc::downgrade(ast),
                        type_deduced.clone(),
                        type_expected.clone(),
                        anno)
    )
  }

  /// Type inference wiring
  /// Generate type equations from node. Each type variable is opposed to some type which we know, or
  /// to Any, if we don't know.
  #[named]
  pub fn generate_equations(&self, module: &Module,
                            eq: &mut Vec<TypeEquation>, ast: &Arc<CoreAst>) -> ErlResult<()> {
    // Recursively descend into AST and visit deepest nodes first
    if let Some(children) = ast.children() {
      for child in children {
        let gen_result = self.generate_equations(module, eq, &child);
        match gen_result {
          Ok(_) => {} // nothing, all good
          Err(err) => { module.add_error(err); }
        }
      }
    }

    match ast.deref() {
      CoreAst::ModuleFuns(list) => {
        for each_fn in list.iter() {
          self.generate_equations(module, eq, &each_fn)?
        }
      }
      CoreAst::Module { .. } => {} // module root creates no equations
      CoreAst::Attributes { .. } => {}
      CoreAst::Lit { .. } => {
        // Literals' type is already known, no equation is created
        // Self::equation(eq, ast, &ty, &value.get_type());
      }
      CoreAst::Var { .. } => {}
      CoreAst::FnDef(fn_def) => {
        self.generate_equations_fndef(eq, ast, fn_def)?;
        // no clauses, functions are single clause, using `case` to branch
        // for fc in &fn_def.clauses {
        //   self.generate_equations_fn_clause(eq, ast, &fc)?
        // }
      }
      CoreAst::Apply(app) => Unifier::generate_equations_apply(eq, ast, app)?,
      CoreAst::Let(letexpr) => {
        Self::equation(eq, ast,
                       &ErlType::TVar(letexpr.ret_ty).into(),
                       &letexpr.in_expr.get_type());
      }
      CoreAst::Case(case) => self.generate_equations_case(eq, ast, case)?,
      CoreAst::BinOp { op: binop, .. } => Unifier::generate_equations_binop(eq, ast, &binop),
      CoreAst::UnOp { op: unop, .. } => {
        // Equation of expression type must match either bool for logical negation,
        // or (int|float) for numerical negation
        Self::equation(eq, ast, &unop.expr.get_type(), &unop.get_type());
        // TODO: Match return type with inferred return typevar?
      }
      CoreAst::List { .. } => {}
      CoreAst::FnRef { .. } => {}
      CoreAst::Tuple { .. } => {}
      CoreAst::PrimOp { op, .. } => Unifier::generate_equations_primop(eq, ast, op),

      CoreAst::Empty => panic!("{}: Called on empty AST", function_name!()),
      _ => {
        println!("{}: Can't process {:?}", function_name!(), ast);
        unreachable!()
      }
    }
    Ok(())
  }

  /// Type inference wiring
  #[named]
  fn generate_equations_primop(eq: &mut Vec<TypeEquation>, ast: &Arc<CoreAst>, op: &PrimOp) {
    // Any value can be raised, no check
    if let PrimOp::Raise { .. } = op {
      println!("TODO: generate equations for primop::Raise")
    } else {
      panic!("{}: Don't know how to process PrimOp {:?}", function_name!(), ast)
    }
  }

  /// Type inference wiring
  /// Generate type equations for CoreAst::BinOp
  fn generate_equations_binop(eq: &mut Vec<TypeEquation>, ast: &Arc<CoreAst>, binop: &&BinaryOperatorExpr) {
    // Check result of the binary operation
    Self::equation(eq, ast, &ErlType::TVar(binop.ty).into(),
                   &binop.get_result_type());

    if let Some(arg_type) = binop.get_arg_type() {
      // Both sides of a binary op must have type appropriate for that op
      Self::equation(eq, ast, &binop.left.get_type(), &arg_type);
      Self::equation(eq, ast, &binop.right.get_type(), &arg_type);
    }
  }

  /// Type inference wiring
  /// Generate type equations for CoreAst::Case and its clauses
  #[named]
  fn generate_equations_case(&self, eq: &mut Vec<TypeEquation>,
                             ast: &Arc<CoreAst>, case: &Case) -> ErlResult<()> {
    // For Case expression, type of case must be union of all clause types
    let all_clause_types = case.clauses.iter()
        .map(|c| c.body.get_type())
        .collect();
    let all_clauses_t = ErlType::union_of(all_clause_types, true);

    for clause in case.clauses.iter() {
      // Clause type must match body type
      Self::equation(eq, ast, &ErlType::TVar(clause.ret_ty).into(),
                     &clause.body.get_type());

      // No check for clause condition, but the clause condition guard must be boolean
      if let Some(guard) = &clause.guard {
        Self::equation(eq, ast, &guard.get_type(),
                       &ErlType::AnyBool.into());
      }
    }

    Self::equation(eq, ast, &ErlType::TVar(case.ret_ty).into(),
                   &all_clauses_t);
    Ok(())
  }

  /// Type inference wiring
  /// Generate type equations for a function definition
  #[named]
  fn generate_equations_fndef(&self, eq: &mut Vec<TypeEquation>,
                              ast: &Arc<CoreAst>, fn_def: &FnDef) -> ErlResult<()> {
    Self::equation(eq, ast, &ErlType::TVar(fn_def.ret_ty).into(),
                   &fn_def.body.get_type());

    // TODO: Exhaustive pattern analysis for function args - in the case statement

    Ok(())
  }

  /// Type inference wiring
  /// Generate type equations for CoreAst::Apply (a function call): Expr(Arg, ...)
  #[named]
  fn generate_equations_apply(eq: &mut Vec<TypeEquation>,
                              ast: &Arc<CoreAst>, app: &Apply) -> ErlResult<()> {
    // The expression we're calling must be something callable, i.e. must match a fun(Arg...)->Ret
    // Produce rule: App.Expr.type <=> fun(T1, T2, ...) -> Ret
    let target_type = app.target.get_type();

    Self::equation(eq, ast,
                   &target_type,
                   &app.get_function_type());

    // The return type of the application (App.Ret) must match the return type of the fun(Args...)->Ret
    // Equation: Application.Ret <=> Expr(Args...).Ret
    // let expr_type: &FunctionType = app.expr_ty.as_function();
    if let ErlType::Fn(fntype) = target_type.deref() {
      Self::equation_anno(eq, ast,
                          &ErlType::TVar(app.ret_ty).into(),
                          &fntype.ret_type,
                          "Match ret type of application with ret type of target".to_string());
    } else {
      panic!("{}: Expecting a Fn type for the application target, got {}", function_name!(), target_type);
    }
    // to do!("Match app.ret with return type of the expr");
    Ok(())
  }
}