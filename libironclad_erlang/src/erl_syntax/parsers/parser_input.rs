//! Contains implementations required for custom nom input to work

use crate::erl_syntax::parsers::token_stream::token::{format_tok_stream, Token};
use crate::project::module::module_impl::ErlModule;
use libironclad_util::source_file::SourceFile;
use nom::Needed;
use std::iter::Enumerate;
use std::mem::size_of;
use std::ops::{RangeFrom, RangeTo};
use std::slice::Iter;

/// The nom-compatible token input
#[derive(Debug, Clone)]
pub struct ParserInput<'a> {
  /// Access to filename and source text, if value is defined
  pub source_file: Option<SourceFile>,
  /// Scope for the parser to update/query
  pub module: ErlModule,
  /// The token stream
  pub tokens: &'a [Token],
}

// /// Wrapper for parser input with parser scope
// pub type ParserInput<'a> = ParserInputT<'a, Option<RootScope>>;

impl<'a> nom::Offset for ParserInput<'a> {
  fn offset(&self, second: &Self) -> usize {
    let fst = self.tokens.as_ptr();
    let snd = second.tokens.as_ptr();

    (snd as usize - fst as usize) / size_of::<Token>()
  }
}

// impl std::fmt::Display for ParserInputImpl<'_> {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     write!(f, "{}", self.as_str())
//   }
// }
//
// impl std::fmt::Debug for ParserInputImpl<'_> {
//   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     write!(f, "ParserInput[ scope={:?}, input={:?} ]", &self.parser_scope, self.input)
//   }
// }
//

impl<'a> ParserInput<'a> {
  /// Calculates offset for second inside `self`
  pub(crate) fn offset_inside(&self, base: &[Token]) -> usize {
    let snd = self.tokens.as_ptr();
    let fst = base.as_ptr();

    if cfg!(debug_assertions) && snd < fst {
      let fst_cut: String = format_tok_stream(self.tokens, 30);
      let snd_cut: String = format_tok_stream(base, 30);
      assert!(
        snd >= fst,
        "snd {:x} must be >= fst {:x}\nfst = {:?}\nsnd = {:?}",
        snd as usize,
        fst as usize,
        fst_cut,
        snd_cut,
      );
    }

    (snd as usize - fst as usize) / size_of::<Token>()
  }

  pub(crate) fn new(source_file: &SourceFile, module: ErlModule, tokens: &'a [Token]) -> Self {
    Self {
      source_file: Some(source_file.clone()),
      module,
      tokens,
    }
  }

  /// Use when you only have slice
  pub fn new_slice(module: ErlModule, tokens: &'a [Token]) -> Self {
    Self { source_file: None, module, tokens }
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.tokens.is_empty()
  }

  //   /// Return a code location
  //   pub(crate) fn loc(&self) -> SourceLoc {
  //     SourceLoc::from_input(self.input)
  //   }
  //
  //   /// Create a parser input with a string slice
  //   pub fn new(source_file: &SourceFile, input: &'a [Token]) -> Self {
  //     Self {
  //       parent_file: Some(source_file.clone()),
  //       parser_scope: ParserScopeImpl::new_empty().into(),
  //       input,
  //     }
  //   }
  //
  //   pub(crate) fn file_name(&self) -> Option<PathBuf> {
  //     self.parent_file.map(|pf| pf.file_name.to_path_buf())
  //     // if let Some(pf) = &self.input.parent_file {
  //     //   Some(pf.file_name.to_path_buf())
  //     // } else {
  //     //   None
  //     // }
  //   }
  //
  /// Clone into a new custom parser input from a str slice. Assert that it belongs to the same input slice.
  pub(crate) fn clone_with_slice(&self, input: &'a [Token]) -> Self {
    Self {
      source_file: None,
      tokens: input,
      module: self.module.clone(),
    }
  }
  //
  //   /// Build a new custom parser input from a loaded source file
  //   pub(crate) fn new_with_scope(
  //     scope: ParserScope,
  //     source_file: &SourceFile,
  //     input: &'a [Token],
  //   ) -> Self {
  //     Self {
  //       parent_file: Some(source_file.clone()),
  //       parser_scope: scope,
  //       input,
  //     }
  //   }
  //
  //   /// Build a new custom parser input from a loaded source file
  //   pub(crate) fn clone_with_input(&self, input: &'a [Token]) -> Self {
  //     Self {
  //       parent_file: self.parent_file.clone(),
  //       parser_scope: self.parser_scope.clone(),
  //       input,
  //     }
  //   }
  //
  //   // /// Build a new custom parser and chain the old to it
  //   // pub(crate) fn clone_nested(&self, input: &str) -> Self {
  //   //   // println!("Parser input clone nested...");
  //   //   ParserInputImpl {
  //   //     parser_scope: self.parser_scope.clone(),
  //   //     input: ParserInputSlice::chain_into_new(&self.input, input),
  //   //     _phantom: Default::default(),
  //   //   }
  //   // }
  //
  //   /// Check whether there's any input remaining
  //   pub(crate) fn is_empty(&self) -> bool {
  //     self.as_str().is_empty()
  //   }
  //
  //   // /// Quick access to last input in chain as `&str`
  //   // #[inline(always)]
  //   // pub fn as_str(&self) -> &'a str {
  //   //   self.input.as_str()
  //   // }
}
//
// impl From<&str> for ParserInputImpl<'_> {
//   fn from(s: &str) -> Self {
//     ParserInputImpl::new_str(s)
//   }
// }
//
// impl nom::Offset for ParserInputImpl<'_> {
//   fn offset(&self, second: &Self) -> usize {
//     // Compare that chain of slices matches in both `self` and `second` and compare that the input
//     // string is the same input string in both.
//     // TODO: It is possible to implement correct offset inside virtual chain of inputs
//     assert_eq!(
//       self.input.parent.as_ptr(),
//       second.input.parent.as_ptr(),
//       "nom::Offset for unrelated slices not implemented (but possible!)"
//     );
//     let self_n = self.as_str().as_ptr() as usize;
//     let second_n = second.as_str().as_ptr() as usize;
//     // println!("Offset for {:x} vs {:x}", self_n, second_n);
//     assert!(
//       second_n >= self_n,
//       "Second input pointer must be greater than the first, when calculating nom::Offset"
//     );
//     second_n - self_n
//   }
// }

// impl Deref for ParserInput<'_> {
//   type Target = str;
//
//   fn deref(&self) -> &Self::Target {
//     self.tokens.iter().next().unwrap()
//   }
// }

impl nom::Slice<RangeFrom<usize>> for ParserInput<'_> {
  fn slice(&self, range: RangeFrom<usize>) -> Self {
    self.clone_with_slice(self.tokens.slice(range))
  }
}

impl nom::Slice<RangeTo<usize>> for ParserInput<'_> {
  fn slice(&self, range: RangeTo<usize>) -> Self {
    self.clone_with_slice(self.tokens.slice(range))
  }
}

impl<'a> nom::InputIter for ParserInput<'a> {
  type Item = Token;
  type Iter = Enumerate<Self::IterElem>;
  type IterElem = std::iter::Cloned<Iter<'a, Token>>;

  #[inline]
  fn iter_indices(&self) -> Self::Iter {
    self.iter_elements().enumerate()
  }
  #[inline]
  fn iter_elements(&self) -> Self::IterElem {
    self.tokens.iter().cloned()
  }
  #[inline]
  fn position<P>(&self, predicate: P) -> Option<usize>
  where
    P: Fn(Self::Item) -> bool,
  {
    self.tokens.iter().position(|b| predicate(b.clone()))
  }
  #[inline]
  fn slice_index(&self, count: usize) -> Result<usize, Needed> {
    if self.tokens.len() >= count {
      Ok(count)
    } else {
      Err(Needed::new(count - self.tokens.len()))
    }
  }
}

impl<'a> nom::InputLength for ParserInput<'a> {
  fn input_len(&self) -> usize {
    self.tokens.len()
  }
}
