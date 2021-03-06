//! Creating ErlErrors from other types
use crate::error::ic_error::IroncladError;
use crate::error::ic_error_category::IcErrorCategory;
use crate::source_loc::SourceLoc;
use libironclad_util::io::file_error::IcFileError;
use std::num::ParseIntError;

impl From<std::io::Error> for IroncladError {
  fn from(value: std::io::Error) -> Self {
    IroncladError::new_type_only(IcErrorCategory::StdIoError(value))
  }
}

impl From<IcFileError> for IroncladError {
  fn from(value: IcFileError) -> Self {
    IroncladError::new_type_only(IcErrorCategory::IcFileError(value))
  }
}

impl From<toml::de::Error> for IroncladError {
  fn from(value: toml::de::Error) -> Self {
    IroncladError::new_type_only(IcErrorCategory::Config(value))
  }
}

impl From<glob::GlobError> for IroncladError {
  fn from(value: glob::GlobError) -> Self {
    IroncladError::new_type_only(IcErrorCategory::Glob(value))
  }
}

impl From<glob::PatternError> for IroncladError {
  fn from(value: glob::PatternError) -> Self {
    IroncladError::new_type_only(IcErrorCategory::GlobPattern(value))
  }
}

impl From<ParseIntError> for IroncladError {
  fn from(pie: ParseIntError) -> Self {
    IroncladError::new(
      IcErrorCategory::ErlangParse,
      SourceLoc::None,
      format!("Cannot parse integer: {}", pie),
    )
  }
}
