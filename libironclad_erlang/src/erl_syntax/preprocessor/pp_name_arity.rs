//! MFArity-like name/arity pair

// /// A key to preprocessor defines dictionary, as defines can coexist with same name but different
// /// number of args
// #[derive(Clone, Eq, PartialEq, Hash)]
// #[deprecated = "use MFArity"]
// pub struct NameArity {
//   /// Name for namearity pair
//   pub name: String,
//   /// The count of arguments
//   pub arity: usize,
// }

// impl NameArity {
//   /// Create a new Name-arity pair
//   pub(crate) fn new(name: &str, arity: usize) -> NameArity {
//     NameArity { name: name.to_string(), arity }
//   }
// }

// impl std::fmt::Debug for NameArity {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     write!(f, "{}/{}", self.name, self.arity)
//   }
// }
