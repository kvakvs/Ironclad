use errloc_macros::errloc;
use crate::project::ErlProject;
use std::sync::{Arc, RwLock};
use crate::stage::file_contents_cache::FileContentsCache;
use std::path::PathBuf;
use std::borrow::{BorrowMut, Borrow};
use crate::erl_error::{ErlResult, ErlError};
use std::collections::HashSet;
use crate::erl_parse;

/// Returns: True if a file was preprocessed
fn preprocess_file(file_name: &PathBuf,
                   hrl_cache: &mut FileContentsCache,
                   erl_cache: &mut FileContentsCache) -> ErlResult<bool> {
    let output = String::new();

    let contents = erl_cache.contents.get(file_name).unwrap(); // trust that file exists
    let (tail, pp_forms) = erl_parse::preprocessor::parse_module(&contents)?;
    println!("\n\
        filename: {}\n\
        PP AST {:?}", file_name.display(), pp_forms);
    if tail.len() > 0 {
        println!("Parse did not succeed. Remaining input: {}", tail);
        return Ok(false)
    }

    // Success: insert new string into preprocessed source cache
    erl_cache.contents.insert(file_name.clone(), output);
    Ok(true)
}

/// Preprocessor stage
/// * Rough pre-parse loaded ERL files, being only interested in preprocessor tokens.
/// * Pre-parse include files AST and paste into include locations.
/// * Drop AST branches covered by the conditional compile directives.
/// Side effects: Updates file contents cache
/// Returns: preprocessed collection of module sources
pub fn run(project: &mut ErlProject,
           file_contents: Arc<RwLock<FileContentsCache>>,
) -> ErlResult<()> {
    let mut hrl_cache = FileContentsCache::new();

    // Take only .erl files
    let mut erl_cache_rw = file_contents.write().unwrap();

    let all_erl_files: HashSet<PathBuf> = erl_cache_rw.contents.keys()
        .into_iter()
        .filter(|path| path.to_string_lossy().ends_with(".erl"))
        .cloned()
        .collect();
    let mut preprocessed_count = 0;

    all_erl_files.into_iter().for_each(
        |path| {
            if preprocess_file(&path,
                               hrl_cache.borrow_mut(),
                               erl_cache_rw.borrow_mut()).unwrap() {
                preprocessed_count += 1;
            }
        });

    println!("Preprocessed {} sources", preprocessed_count);

    // Drop .hrl file contents after preprocess stage
    // let arc_hrl_cache = Arc::new(RwLock::new(hrl_cache));
    Ok(())
}
