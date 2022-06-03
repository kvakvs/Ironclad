//! Contains code to wrap `String` with macro substitution features

use crate::typing::scope::Scope;
use std::cell::RefCell;
use std::fmt::Formatter;

/// A string container which can have its string replaced entirely.
pub struct MacroString {
  /// Replaceable cell containing text for the node. If it contains preprocessor definitions,
  /// values will be substituted before further processing
  pub text: RefCell<String>,
}

impl std::fmt::Debug for MacroString {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "\"{}\"", self.text.borrow())
  }
}

impl Clone for MacroString {
  fn clone(&self) -> Self {
    return MacroString::new_string(self.text.borrow().clone());
  }
}

impl MacroString {
  /// Create with no text
  pub fn new_empty() -> Self {
    Self { text: RefCell::new(String::default()) }
  }

  /// Creates a macro-string
  pub fn new(text: &str) -> Self {
    Self { text: text.to_string().into() }
  }

  /// Creates a macro-string with `String`
  pub fn new_string(text: String) -> Self {
    Self { text: text.into() }
  }

  /// Scan string for `?macro`s, and replace if scope contains the macros.
  pub fn update(&self, _scope: &Scope) {
    // let text2 = self.text.replace("?MACRO");
    // self.text.(String::new())
  }
}
