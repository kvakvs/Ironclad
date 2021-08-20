//! Erlang literals, values fully known at compile time
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;

use crate::typing::erl_type::ErlType;

/// An Erlang literal, a value fully known at compile time
#[derive(Clone)]
pub enum Literal {
  // TODO: Big integer
  /// Small enough to fit into a machine word
  Integer(isize),

  /// A 8-byte wide float
  Float(f64),

  /// Atom literal, also includes atoms 'true' and 'false'
  Atom(String),
  // TODO: String/list lit, tuple lit, map lit, binary lit, etc

  /// A boolean value true or false atom, is-a(Atom)
  Bool(bool),

  // Cannot have runtime values as literals
  // Pid,
  // Reference,

  /// A list of literals
  List {
    /// List elements
    elements: Vec<Literal>,
    /// Optional tail element or None if NIL
    tail: Option<Box<Literal>>,
  },

  /// An empty list
  Nil,

  /// A list containing only unicode codepoints is-a(List)
  String(String),

  /// A tuple of literals
  Tuple(Vec<Literal>),
}

impl Hash for Literal {
  fn hash<H: Hasher>(&self, state: &mut H) {
    match self {
      Literal::Integer(n) => {
        'i'.hash(state);
        n.hash(state);
      }
      Literal::Float(f) => {
        'f'.hash(state);
        format!("{}", f).hash(state);
      }
      Literal::Atom(a) => {
        'a'.hash(state);
        a.hash(state);
      }
      Literal::Bool(b) => {
        'b'.hash(state);
        b.hash(state);
      }
      Literal::List { elements, .. } => {
        'L'.hash(state);
        elements.hash(state);
      }
      Literal::Nil => { "[]".hash(state); }
      Literal::String(s) => {
        's'.hash(state);
        s.hash(state);
      }
      Literal::Tuple(elements) => {
        'T'.hash(state);
        elements.hash(state);
      }
    }
  }

  fn hash_slice<H: Hasher>(data: &[Self], state: &mut H) where Self: Sized {
    data.iter().for_each(|d| d.hash(state))
  }
}

impl Literal {
  /// Retrieves a type of a literal
  pub fn get_type(&self) -> ErlType {
    match self {
      Literal::Integer(i) => ErlType::Integer(*i),
      Literal::Float(_) => ErlType::Float,
      Literal::Atom(s) => ErlType::Atom(s.clone()),
      Literal::Bool(_) => ErlType::AnyBool,
      // Cannot have runtime values as literals
      // ErlLit::Pid => ErlType::Pid,
      // ErlLit::Reference => ErlType::Reference,
      Literal::List { elements, .. } => {
        // List type is union of all element types
        ErlType::List(Box::new(ErlType::union_of_literal_types(elements)))
      }
      Literal::Nil => ErlType::AnyList,
      Literal::String(_) => ErlType::String, // is-a(list(char))
      Literal::Tuple(items) => {
        ErlType::Tuple(items.iter()
            .map(|it| it.get_type())
            .collect())
      }
    }
  }
}

impl Eq for Literal {}

impl PartialEq for Literal {
  fn eq(&self, other: &Self) -> bool {
    match (self, other) {
      (Literal::Integer(a), Literal::Integer(b)) => a == b,
      (Literal::Float(a), Literal::Float(b)) => (a - b).abs() <= f64::EPSILON,
      (Literal::Atom(a), Literal::Atom(b)) => a == b,
      (Literal::Bool(a), Literal::Bool(b)) => a == b,
      (Literal::List { elements: a, .. }, Literal::List { elements: b, .. }) => a == b,
      (Literal::String(a), Literal::String(b)) => a == b,
      (Literal::Tuple(a), Literal::Tuple(b)) => a == b,
      _ => false,
    }
  }
}

impl PartialOrd<Self> for Literal {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Literal {
  fn cmp(&self, other: &Self) -> Ordering {
    let self_order = self.get_type().get_order();
    let other_order = other.get_type().get_order();
    let order = self_order.cmp(&other_order);
    match order {
      Ordering::Less | Ordering::Greater => order,
      Ordering::Equal => self.cmp_same_type(other),
    }
  }
}

impl Literal {
  /// Compares two literals of same kind, otherwise general ordering applies
  pub fn cmp_same_type(&self, other: &Literal) -> Ordering {
    match (self, other) {
      (Literal::Integer(a), Literal::Integer(b)) => a.cmp(b),
      (Literal::Float(a), Literal::Float(b)) => {
        if (a - b).abs() <= f64::EPSILON {
          Ordering::Equal
        } else {
          a.partial_cmp(b).unwrap()
        }
      }
      (Literal::Atom(a), Literal::Atom(b)) => a.cmp(b),
      (Literal::Bool(a), Literal::Bool(b)) => a.cmp(b),
      (Literal::List { elements: a, .. }, Literal::List { elements: b, .. }) => a.cmp(b),
      (Literal::String(a), Literal::String(b)) => a.cmp(b),
      (Literal::Tuple(a), Literal::Tuple(b)) => a.cmp(b),
      _ => unreachable!("Can't compare {} vs {}, only same type allowed in this function", self, other)
    }
  }
}