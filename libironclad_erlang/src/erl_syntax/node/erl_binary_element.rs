//! Binary element is an item in a binary expression or a binary builder.
//! This models the binary syntax of Erlang: <<1, 2, 3:8/bits, Variable, (function()):16 ...>>

use crate::erl_syntax::erl_ast::AstNode;
use crate::source_loc::SourceLoc;
use libironclad_util::pretty::Pretty;

/// Added to `BinaryTypeSpecifier` after `:` to specify the bit width.
/// Sometimes bit width is known at compile time and sometimes is not.
#[allow(missing_docs)]
#[derive(Debug)]
pub enum ValueWidth {
  /// Bit width is known at compile time, and is an integer literal (other literals not acceptable)
  Literal(usize),
  /// Expression should resolve to an integer and will define bit width
  Expr(AstNode),
  /// Default is chosen by the libironclad automatically to fit the chosen value type
  DefaultWidth,
}

/// Added to `BinaryTypeSpecifier` to define how to insert a value
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum ValueType {
  Integer,
  Float,
  Bytes,
  // synonym: Binary,
  Bitstring,
  // synonym: Bits,
  Utf8,
  Utf16,
  Utf32,
}

/// Added to `BinaryTypeSpecifier` to define presence of sign-bit and special rules for negative values
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum ValueSignedness {
  Signed,
  Unsigned,
}

/// Added to `BinaryTypeSpecifier` to define byte order
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum ValueEndianness {
  Big,
  Little,
  Native,
}

/// Type specifier, one element added to any binary expression element after a `/`:
/// like so `X:4/little-signed-integer-unit:8`
#[allow(missing_docs)]
#[derive(Clone, Debug)]
pub enum TypeSpecifier {
  Type(ValueType),
  Signedness(ValueSignedness),
  Endianness(ValueEndianness),
  /// Default: byte=8, float=64, bytes or binary=entire size
  Unit(usize),
}

impl std::fmt::Display for TypeSpecifier {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TypeSpecifier::Type(t) => match t {
        ValueType::Integer => write!(f, "integer"),
        ValueType::Float => write!(f, "float"),
        ValueType::Bytes => write!(f, "bytes"),
        ValueType::Bitstring => write!(f, "bitstring"),
        ValueType::Utf8 => write!(f, "utf8"),
        ValueType::Utf16 => write!(f, "utf16"),
        ValueType::Utf32 => write!(f, "utf32"),
      },
      TypeSpecifier::Signedness(s) => match s {
        ValueSignedness::Signed => write!(f, "signed"),
        ValueSignedness::Unsigned => write!(f, "unsigned"),
      },
      TypeSpecifier::Endianness(e) => match e {
        ValueEndianness::Big => write!(f, "big"),
        ValueEndianness::Little => write!(f, "little"),
        ValueEndianness::Native => write!(f, "native"),
      },
      TypeSpecifier::Unit(u) => write!(f, "unit:{}", u),
    }
  }
}

/// An item in a binary expression or a binary builder
#[derive(Debug)]
pub struct BinaryElement {
  /// Where in the code
  pub location: SourceLoc,
  /// The value part: an expression, or a literal, etc.
  pub value: AstNode,
  /// Bit width for the value, if specified after `:`. Zero means "not specified".
  pub width: ValueWidth,
  /// Elements of type spec added after `/` like so: `X:4/little-signed-integer-unit:8`
  pub type_specs: Vec<TypeSpecifier>,
}

impl BinaryElement {
  /// Creates a new freshly parsed element of a binary expression
  pub(crate) fn new(
    location: SourceLoc,
    value: AstNode,
    width: ValueWidth,
    type_specs: Vec<TypeSpecifier>,
  ) -> Self {
    Self { location, value, width, type_specs }
  }
}

impl std::fmt::Display for BinaryElement {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    self.value.fmt(f)?;
    match &self.width {
      ValueWidth::Literal(w) => write!(f, ":{}", w)?,
      ValueWidth::Expr(e) => write!(f, ":({})", e)?,
      ValueWidth::DefaultWidth => {}
    }
    if !self.type_specs.is_empty() {
      write!(f, "/")?;
      Pretty::display_separated(self.type_specs.iter(), "-", f)?;
    }
    Ok(())
  }
}
