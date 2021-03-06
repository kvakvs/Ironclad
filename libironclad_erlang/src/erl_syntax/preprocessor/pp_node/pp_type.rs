//! Preprocessor node type enum

use crate::erl_syntax::erl_ast::AstNode;
use crate::erl_syntax::node::erl_record::RecordField;
use crate::erl_syntax::parsers::token_stream::token::Token;
use crate::typing::erl_type::ErlType;
use libironclad_util::mfarity::MFArity;

/// Preprocessor data nodes, are produced during the `ErlModule::stage_preprocess` stage.
/// They are not stored anywhere in the final AST.
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum PreprocessorNodeType {
  /// Opens a module
  ModuleName { name: String },
  /// Specific directive: -include("path").
  Include(String),
  /// Specific directive: -include_lib("path").
  IncludeLib(String),
  /// Define directive: `-define(NAME)` or `-define(NAME, TEXT)` or `-define(NAME(ARGS), TEXT)`.
  Define {
    /// Macro name
    name: String,
    /// Args if specified, different arity macros do not conflict each with other
    args: Vec<String>,
    /// Body if specified, any tokens till the delimiter: `) . NEWLINE`
    body: Vec<Token>,
  },
  /// Specific directive: -undef(NAME). removes a named macro definition
  Undef(String),
  /// Proceed interpreting AST nodes if the named macro is defined
  Ifdef {
    /// The condition to check
    macro_name: String,
  },
  /// Proceed interpreting AST nodes if the named macro is NOT defined
  Ifndef {
    /// The condition to check
    macro_name: String,
  },
  /// If(expression) stores an expression which must resolve to a constant value otherwise compile
  /// error will be triggered.
  If {
    /// The condition to check
    cond: AstNode,
  },
  /// Produce a libironclad error
  Error(String),
  /// Produce a libironclad warning
  Warning(String),
  // /// Nested included file
  // IncludedFile {
  //   /// Filename for this included file
  //   filename: PathBuf,
  //   /// Preprocessor sub-tree to descend into the includefile
  //   tokens: Vec<Token>,
  // },
  // Temporary nodes, appear during parsing and should never appear into the final AST output.
  // These values never leave the parser module.
  /// `-else.` node
  Else,
  ///`-endif` node
  Endif,
  /// `-elif(...).` node
  ElseIf { cond: AstNode },
  /// A generic attribute with tag and one optional value `- <TAG> ( <VALUE> ).`
  Attr {
    /// Attribute tag in `- <TAG> ().`
    tag: String,
    /// Constant expression value inside parentheses for `-TAG( EXPR )`
    term: Option<AstNode>,
  },
  /// List of exported FunArities for local functions
  Export {
    /// The contents of `-export([...]).`
    fun_arities: Vec<MFArity>,
  },
  /// List of exported FunArities for types
  ExportType {
    /// The contents of `-export_type([...]).`
    type_arities: Vec<MFArity>,
  },
  /// List of imported FunArities from a module
  Import {
    /// From module
    module: String,
    /// The contents of `-import(MODULE, [...]).`
    fun_arities: Vec<MFArity>,
  },
  /// Defines a new type, globally available in the module
  NewType {
    /// Custom type name
    name: String,
    /// Type parameter variables
    vars: Vec<String>,
    /// Type definition
    ty: ErlType,
  },
  /// A new record definition, created by `-record(name, {fields,...}).` attribute
  NewRecord {
    /// Record tag
    tag: String,
    /// Fields with optional initializers and optional type ascriptions
    fields: Vec<RecordField>,
  },
  /// A function spec, written as `-spec myfun(...) -> <ret type> when ... <optional when>.`
  FnSpec {
    /// The function name and arity, module as None
    funarity: MFArity,
    /// Type for all function clauses
    spec: ErlType,
  },
}
