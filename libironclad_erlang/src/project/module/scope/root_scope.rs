//! Similar to `Scope` but contains root definitions for entire module

use crate::erl_syntax::erl_ast::ast_iter::IterableAstNodeT;
use crate::erl_syntax::erl_ast::node_impl::AstNodeType;
use crate::erl_syntax::erl_ast::AstNode;
use crate::project::module::scope::mod_attr::ModuleAttributes;
use crate::typing::erl_type::ErlType;
use libironclad_util::mfarity::MFArity;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct RootScopeImpl {
  /// Contains definitions, added by `-spec` attribute
  function_specs: RwLock<HashMap<MFArity, Arc<ErlType>>>,

  /// Functions can only be found on the module root scope (but technically can be created in the
  /// other internal scopes too)
  function_defs: RwLock<HashMap<MFArity, AstNode>>,

  /// Types defined in the global module scope. Using typename/arity as key in type hierarchy
  typedefs: RwLock<HashMap<MFArity, Arc<ErlType>>>,
  /// Collection of all custom attributes coming in form of `- <TAG> ( <EXPR> ).` tag is key in this
  /// collection and not unique.
  attributes: RwLock<HashMap<String, Arc<ModuleAttributes>>>,
}

/// Alias type for `Arc<>`
pub type RootScope = Arc<RootScopeImpl>;

impl Default for RootScopeImpl {
  fn default() -> Self {
    RootScopeImpl {
      function_specs: Default::default(),
      function_defs: Default::default(),
      typedefs: Default::default(),
      attributes: Default::default(),
    }
  }
}

impl std::fmt::Display for RootScopeImpl {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let attrs_fmt = if let Ok(r_attrs) = self.attributes.read() {
      r_attrs
        .iter()
        .map(|(key, a)| format!("{}={:?}", key, a))
        .collect::<Vec<String>>()
        .join(", ")
    } else {
      panic!("Can't lock Scope to print vars")
    };
    let funs_fmt = if let Ok(r_funs) = self.function_defs.read() {
      r_funs
        .iter()
        .map(|fnc| format!("{}", fnc.0))
        .collect::<Vec<String>>()
        .join(", ")
    } else {
      panic!("Can't lock Scope to print funs")
    };
    write!(f, "RootScope{{ funs [{}], attrs [{}] }}", funs_fmt, attrs_fmt)
  }
}

impl RootScopeImpl {
  /// Retrieve named attributes
  pub fn get_attr(&self, attr_tag: &str) -> Option<Arc<ModuleAttributes>> {
    if let Ok(r_attrs) = self.attributes.read() {
      r_attrs.get(&attr_tag.to_string()).map(|ma| ma.clone())
    } else {
      panic!("Can't lock scope to read attributes")
    }
  }

  /// Retrieve named function spec
  pub fn get_spec(&self, mfa: &MFArity) -> Option<Arc<ErlType>> {
    if let Ok(r_specs) = self.function_specs.read() {
      r_specs.get(&mfa).map(|spec| spec.clone())
    } else {
      panic!("Can't lock scope to lookup function specs")
    }
  }

  /// Attempt to find a function in the scope, or delegate to the parent scope
  pub(crate) fn find_fn_ast(&self, mfa: &MFArity) -> Option<AstNode> {
    if let Ok(r_funs) = self.function_defs.read() {
      match r_funs.get(mfa) {
        Some(val) => {
          if val.is_fn_def() {
            return Some(val.clone());
          }
          panic!("Only FnDef AST nodes must be stored in module scope")
        }
        None => None,
      }
    } else {
      panic!("Can't lock RootScope functions for lookup")
    }
  }

  /// Add a function by MFA and its type
  pub(crate) fn add_fn(&self, mfa: &MFArity, ast: AstNode) {
    if let Ok(mut w_funs) = self.function_defs.write() {
      w_funs.insert(mfa.clone(), ast);
    } else {
      panic!("Can't lock scope to update functions")
    }
  }

  /// Recursive descend into AST saving FnDef nodes
  pub fn update_from_ast(&mut self, ast: &AstNode) {
    if let AstNodeType::FnDef(fndef) = &ast.content {
      self.add_fn(&fndef.funarity, ast.clone());
    }

    if let Some(children) = ast.children() {
      for c in children {
        self.update_from_ast(&c)
      }
    }
  }
}