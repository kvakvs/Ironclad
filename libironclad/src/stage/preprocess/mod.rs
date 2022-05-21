//! Preprocess Stage - parses and interprets the Erlang source and gets rid of -if/-ifdef/-ifndef
//! directives, substitutes HRL files contents in place of -include/-include_lib etc.
pub mod pp_scope;
pub mod pp_define;

use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use nom::{Finish};

use crate::erl_error::{ErlError, ErlErrorType, ErlResult};
use crate::preprocessor::nom_parser::{PpAstParserResult, PreprocessorParser};
use crate::project::ErlProject;
use crate::project::source_file::SourceFile;
use crate::source_loc::{SourceLoc};
use crate::stage::file_contents_cache::FileContentsCache;
use crate::preprocessor::syntax_tree::pp_ast::{PpAst, PpAstCache};
use crate::stage::preprocess::pp_scope::PreprocessorScope;

/// Preprocessor state with AST cache, macro definitions, etc
pub struct ErlPreprocessStage {
  /// For headers included more than once, parse them and cache here for reinterpretation as needed
  ast_cache: Arc<Mutex<PpAstCache>>,

  file_cache: Arc<Mutex<FileContentsCache>>,

  /// Contains preprocessor definitions from config, from command line or from the file. Evolves as
  /// the parser progresses through the file and encounters new preprocessor directives.
  scope: Arc<PreprocessorScope>,
}

impl ErlPreprocessStage {
  /// Split input file into fragments using preprocessor directives as separators
  pub fn from_source_file(source_file: &Arc<SourceFile>) -> ErlResult<Arc<PpAst>> {
    let input = &source_file.text;
    Self::from_source(input)
  }

  /// Split input file into fragments using preprocessor directives as separators
  pub fn from_source(input: &str) -> ErlResult<Arc<PpAst>> {
    Self::parse_helper(input, PreprocessorParser::parse_module)
  }

  /// Parse AST using provided parser function, check that input is consumed, print some info.
  /// The parser function must take `&str` and return `Arc<PpAst>` wrapped in a `ParserResult`
  pub fn parse_helper<Parser>(input: &str, parser: Parser) -> ErlResult<Arc<PpAst>>
    where Parser: Fn(&str) -> PpAstParserResult
  {
    let parse_result = parser(input).finish();

    #[cfg(debug_assertions)]
    if parse_result.is_err() {
      println!("NomError: {:?}", parse_result);
    }

    match parse_result {
      Ok((tail, ast)) => {
        assert!(tail.trim().is_empty(),
                "Preprocessor: Not all input was consumed by parse.\n
                \tTail: «{}»\n
                \tAst: {}", tail, ast);
        Ok(ast)
      }
      Err(err) => Err(ErlError::from_nom_error(input, err)),
    }
  }

  /// Returns: True if a file was preprocessed
  fn preprocess_file(&mut self, file_name: &Path) -> ErlResult<()> {
    // trust that file exists
    let contents = {
      let file_cache1 = self.file_cache.lock().unwrap();
      file_cache1.all_files
          .get(file_name).unwrap()
          .clone()
    };

    // If cached, try get it, otherwise parse and save
    let ast_tree = {
      let mut ast_cache = self.ast_cache.lock().unwrap();
      match ast_cache.items.get(file_name) {
        Some(ast) => ast.clone(),
        None => {
          // Parse and cache
          let ast = Self::from_source_file(&contents)?;
          // Save to preprocessor AST cache
          ast_cache.items.insert(file_name.to_path_buf(), ast.clone());
          ast
        }
      }
    };

    let pp_ast = self.interpret_pp_ast(&contents, &ast_tree)?;

    // TODO: Output preprocessed source as iolist, and stream-process in Erlang parser? to minimize the copying
    let output: String = pp_ast.to_string();

    { // Success: insert new string into preprocessed source cache
      let mut file_cache2 = self.file_cache.lock().unwrap();
      file_cache2.update_source_text(file_name, output);
    }

    // Cleanup
    Ok(())
  }
}

fn interpret_include_directive(source_file: &SourceFile,
                               node: &Arc<PpAst>,
                               ast_cache: Arc<Mutex<PpAstCache>>,
                               file_cache: Arc<Mutex<FileContentsCache>>) -> ErlResult<Arc<PpAst>> {
  match node.deref() {
    // Found an attr directive which is -include("something")
    // TODO: Refactor into a outside function with error handling
    PpAst::IncludeLib(path)
    | PpAst::Include(path) => {
      // Take source file's parent dir and append to it the include path (unless it was absolute?)
      let source_path = &source_file.file_name;

      let include_path0 = PathBuf::from(path);
      let include_path = if include_path0.is_absolute()
      { include_path0 } else { source_path.parent().unwrap().join(include_path0) };

      // TODO: Path resolution relative to the file path
      let mut ast_cache1 = ast_cache.lock().unwrap();
      let find_result = ast_cache1.items.get(&include_path);

      match find_result {
        None => {
          let include_source_file = {
            let mut file_cache1 = file_cache.lock().unwrap();
            file_cache1.get_or_load(&include_path).unwrap()
          };
          let ast_tree = ErlPreprocessStage::from_source_file(&include_source_file).unwrap();

          // let mut ast_cache1 = ast_cache.lock().unwrap();
          ast_cache1.items.insert(include_path.clone(), ast_tree.clone());

          Ok(PpAst::new_included_file(&include_path, ast_tree))
        }
        Some(arc_ast) => {
          let result = PpAst::new_included_file(&include_path, arc_ast.clone());
          Ok(result)
        }
      }
    }
    _ => Ok(node.clone())
  }
}

impl ErlPreprocessStage {
  /// This is called for each Preprocessor AST node to make the final decision whether the node
  /// is passed into the output or replaced with a SKIP. "Scope" is global for module and as the
  /// interpretation goes top to bottom, the scope is updated globally and is not nested inside
  /// ifdef/if blocks.
  fn interpret_pp_rule(&mut self,
                       node: &Arc<PpAst>,
                       source_file: &SourceFile,
                       nodes_out: &mut Vec<Arc<PpAst>>,
                       warnings_out: &mut Vec<ErlError>,
                       errors_out: &mut Vec<ErlError>) -> ErlResult<()> {
    // First process ifdef/if!def/else/endif
    match node.deref() {
      PpAst::File(nodes) => {
        for n in nodes {
          self.interpret_pp_rule(n, source_file, nodes_out, warnings_out, errors_out)?;
        }
      }
      PpAst::IncludedFile { ast, .. } => {
        self.interpret_pp_rule(ast, source_file,
                               nodes_out, warnings_out, errors_out)?;
      }
      PpAst::IfdefBlock { macro_name, cond_true, cond_false } => {
        if self.scope.is_defined(macro_name) {
          if let Some(nodes) = cond_true {
            nodes_out.extend(nodes.iter().cloned());
          }
        } else {
          if let Some(nodes) = cond_false {
            nodes_out.extend(nodes.iter().cloned());
          }
        }
      }
      PpAst::Text(_) => nodes_out.push(node.clone()),
      PpAst::EmptyText => {} // skip
      PpAst::Include(arg) => {
        println!("TODO: Include '{}'", arg)
      }
      PpAst::IncludeLib(arg) => {
        println!("TODO: IncludeLib '{}'", arg)
      }
      PpAst::Define { name, args, body } => {
        self.scope = self.scope.define(name, args.clone(), body.clone());
      }
      PpAst::DefineFun { name, args, body } => {
        self.scope = self.scope.define(name, Some(args.clone()), Some(body.clone()));
      }
      PpAst::Undef(_) => {}
      PpAst::IfBlock { .. } => {}
      PpAst::Error(msg) => {
        errors_out.push(ErlError::new(ErlErrorType::Preprocessor,
                                      SourceLoc::None,
                                      msg.clone()));
      }
      PpAst::Warning(_) => {}
      _ => {}
    }

    Ok(())
  }

  /// Create a preprocess state struct for processing a file.
  /// Preprocessor symbols are filled from the command line and project TOML file settings.
  pub fn new(ast_cache: &Arc<Mutex<PpAstCache>>,
             file_cache: &Arc<Mutex<FileContentsCache>>,
             scope: Arc<PreprocessorScope>) -> Self {
    Self {
      ast_cache: ast_cache.clone(),
      file_cache: file_cache.clone(),
      scope,
    }
  }

  /// Interpret parsed attributes/preprocess directives from top to bottom
  /// - Exclude ifdef/if/ifndef sections where the condition check fails
  /// - Load include files and paste them where include directive was found. Continue interpretation.
  /// - Substitute macros.
  ///
  /// Return: a new preprocessed string joined together.
  fn interpret_pp_ast(&mut self,
                      source_file: &SourceFile,
                      ast_tree: &Arc<PpAst>) -> ErlResult<Arc<PpAst>> {
    let mut nodes_out: Vec<Arc<PpAst>> = Vec::default();
    let mut warnings_out: Vec<ErlError> = Vec::default();
    let mut errors_out: Vec<ErlError> = Vec::default();

    self.interpret_pp_rule(ast_tree, source_file,
                           &mut nodes_out, &mut warnings_out, &mut errors_out)?;

    if !errors_out.is_empty() {
      Err(ErlError::multiple(errors_out))
    } else if !warnings_out.is_empty() {
      Err(ErlError::multiple_warnings(warnings_out))
    } else {
      Ok(PpAst::File(nodes_out).into())
    }
  }

  /// Stage 1 - Preprocessor stage
  /// ----------------------------
  /// * Preparse loaded ERL files ignoring the syntax only paying attention to preprocess tokens.
  /// * Preparse include files AST and paste preprocess AST into include locations.
  /// * Drop AST branches covered by the conditional compile directives.
  ///
  /// Side effects: Updates file contents cache
  /// Returns preprocessed collection of module sources
  pub fn run(project: &mut ErlProject,
             file_cache: Arc<Mutex<FileContentsCache>>,
  ) -> ErlResult<Arc<Mutex<PpAstCache>>> {
    let ast_cache = Arc::new(Mutex::new(PpAstCache::default()));

    // Take only .erl files
    let all_files: Vec<PathBuf> = {
      let file_cache_r = file_cache.lock().unwrap();
      file_cache_r.all_files.keys().cloned().collect()
    };

    let mut preprocessed_count = 0;

    let erl_files: Vec<PathBuf> = all_files.into_iter()
        .filter(|path| path.to_string_lossy().ends_with(".erl"))
        .collect();

    for path in erl_files.iter() {
      // For all input files, run preprocess parse and interpred the preprocess directives
      // Loaded and parsed HRL files are cached to be inserted into every include location
      // Create starting scope (from project settings and command line)
      let starting_scope = project.get_preprocessor_scope(&path);
      let mut stage = ErlPreprocessStage::new(&ast_cache, &file_cache, starting_scope);
      stage.preprocess_file(&path)?;
      preprocessed_count += 1;
    }

    let cached_ast_trees_count = {
      let ast_cache_r = ast_cache.lock().unwrap();
      ast_cache_r.items.len()
    };

    println!("Preprocessed {} sources, {} includes",
             preprocessed_count,
             cached_ast_trees_count);
    Ok(ast_cache)
  }
}
