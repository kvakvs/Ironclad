//! Postprocesses an AST tree. Takes a tree in, replaces some nodes, and returns a new tree.

use crate::erlang::syntax_tree::erl_ast::ErlAst;
use crate::erl_error::{ErlResult};
use crate::erlang::syntax_tree::erl_ast_iter::AstChild;
use crate::project::module::Module;

impl Module {
  /// Run the post-parse analysis. Relies upon results from Self::postprocess_ast_readonly().
  /// Given a fresh parsed and processed Erlang AST, go through it once more and edit some nodes.
  /// * In function applications replace atom function names with function pointers
  pub fn postprocess_ast(&self, ast: &ErlAst) -> ErlResult<()> {
    if let Some(children) = ast.children() {
      for child in children {
        match child {
          AstChild::Ref(c) => self.postprocess_ast(c)?,
          AstChild::RefCell(refc) => self.postprocess_ast(&refc)?,
        }
      }
    }

    match ast {
      ErlAst::Apply(_loc, app) => {
        app.postprocess_edit_node(self)?;
      }
      ErlAst::FunctionDef { fn_def, .. } => {
        for fc in &fn_def.clauses {
          for arg in &fc.args {
            self.postprocess_ast(&arg)?;
          }
          self.postprocess_ast(&fc.body)?;
        }
      }
      _ => {}
    }

    Ok(())
  }
}