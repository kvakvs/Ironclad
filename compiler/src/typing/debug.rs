//! Adds debug printing for Type trees in a somewhat more compact way

use std::fmt;
use crate::typing::erl_type::{ErlType, MapField, RecordField};
use crate::typing::typevar::TypeVar;
use crate::typing::equation::TypeEquation;

impl fmt::Debug for ErlType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      ErlType::Union(members) => {
        write!(f, "∪")?;
        f.debug_list().entries(members).finish()
      }
      ErlType::None => write!(f, "∅"),
      ErlType::Any => write!(f, "∀"),
      ErlType::TVar(v) => write!(f, "{}", v.to_string()),
      ErlType::Number => write!(f, "ℝ"),
      ErlType::Integer => write!(f, "ℤ"),
      ErlType::Float => write!(f, "ℚ"),
      ErlType::List(t) => write!(f, "list({:?})", t),
      ErlType::String => write!(f, "str"),
      ErlType::Tuple(t) => write!(f, "tuple({:?})", t),
      ErlType::Record { tag, fields } => {
        let mut d = f.debug_tuple(&tag);
        fields.iter().for_each(|f| {
          d.field(f);
        });
        d.finish()
      }
      ErlType::Map(fields) => {
        let mut d = f.debug_map();
        fields.iter().for_each(|f| {
          d.entry(&f.key, &f.ty);
        });
        d.finish()
      }
      ErlType::Atom => write!(f, "atom"),
      ErlType::Bool => write!(f, "bool"),
      ErlType::Pid => write!(f, "pid"),
      ErlType::Reference => write!(f, "ref"),
      ErlType::BinaryBits => write!(f, "bits"),
      ErlType::Binary => write!(f, "bin"),
      ErlType::Literal(lit) => write!(f, "{:?}", lit),
      ErlType::Function { name, arg_ty, ret } => {
        match name {
          None => write!(f, "fun(")?,
          Some(n) => write!(f, "{}", n)?,
        }

        let mut d = f.debug_tuple("arg");
        arg_ty.iter().for_each(|argt| {
          d.field(argt);
        });
        d.finish()?;

        match name {
          None => write!(f, "→ {:?})", ret),
          Some(_) => write!(f, "→ {:?}", ret),
        }
      }
    }
  }
}

impl fmt::Debug for RecordField {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

impl fmt::Debug for MapField {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

impl fmt::Debug for TypeVar {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

impl fmt::Debug for TypeEquation {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?} ↔ {:?}", self.left, self.right)
  }
}