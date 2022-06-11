//! File contents cache stores all loaded files in memory
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::error::ic_error::IroncladResult;
use crate::source_file::{SourceFile, SourceFileImpl};
use crate::stats::cache_stats::CacheStats;
use crate::stats::io_stats::IOStats;
use std::sync::{Arc, RwLock};

/// Contains loaded files ready for parsing by the preprocessor.
/// More files will be added in preprocess stage, as include directives are parsed
#[derive(Default)]
pub struct FileCacheImpl {
  /// File contents stored here
  pub all_files: BTreeMap<PathBuf, SourceFile>,
  io_stats: IOStats,
  file_cache_stats: CacheStats,
}

/// Wrapper for shared access
pub type FileCache = Arc<RwLock<FileCacheImpl>>;

impl FileCacheImpl {
  /// Create a new file cache with stats attached
  pub fn new(io_stats: IOStats, file_cache_stats: CacheStats) -> FileCache {
    let c = FileCacheImpl {
      all_files: Default::default(),
      io_stats,
      file_cache_stats,
    };
    RwLock::new(c).into()
  }

  /// Load file contents, store entire contents in the hashmap
  pub fn preload_file(&mut self, file_name: &Path) -> IroncladResult<SourceFile> {
    println!("Attempt to load file: {:?}", file_name);

    let contents = std::fs::read_to_string(file_name)?;

    if let Ok(mut w_io_stats) = self.io_stats.write() {
      w_io_stats.files_read += 1;
      w_io_stats.bytes_read += contents.len();
    } else {
      panic!("Can't lock iostats for update")
    }

    let src_file = SourceFileImpl::new(file_name, contents);
    self
      .all_files
      .insert(file_name.to_path_buf(), src_file.clone());
    Ok(src_file)
  }

  /// Retrieve cached file contents or attempt to load (and update the cache)
  /// TODO: Cloning of strings is bad
  pub fn get_or_load(&mut self, file_name: &Path) -> IroncladResult<SourceFile> {
    let canon_path = file_name.canonicalize().unwrap();

    match self.all_files.get(&canon_path) {
      None => {
        if let Ok(mut w_cache_stats) = self.file_cache_stats.write() {
          w_cache_stats.misses += 1;
        } else {
          panic!("Can't lock self.file_cache_stats for update")
        }
        let src_file = self.preload_file(&canon_path)?;
        Ok(src_file)
      }

      Some(contents) => {
        if let Ok(mut w_cache_stats) = self.file_cache_stats.write() {
          w_cache_stats.hits += 1;
        } else {
          panic!("Can't lock self.file_cache_stats for update")
        }
        Ok(contents.clone())
      }
    }
  }

  /// As source file text is read only, we replace.
  /// The parse trees referring the the old source file will retain their Arc<> to the old version
  pub fn update_source_text(&mut self, file_name: &Path, new_text: String) {
    let new_source_file = SourceFileImpl::new(file_name, new_text);
    self
      .all_files
      .insert(file_name.to_path_buf(), new_source_file);
  }
}
