use std::path::PathBuf;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod config;
pub mod error;
mod toml_diag;

pub use error::{BuildErrors, NyxError, SourceFile, Span, TemplateError};

/// Inputs to a full build.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOptions {
    /// The site root, containing the nyx.toml, content, templates and static.
    pub root: PathBuf,
    /// Overrides output_dir from nyx.toml. An explicit override may point outside the site root.
    pub out_dir: Option<PathBuf>,
}

impl BuildOptions {
    /// Options to build the site at root with its configured output directory.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            out_dir: None,
        }
    }
}

pub struct BuildReport {
    pub pages: usize,
    pub posts: usize,
    pub static_files: usize,
    pub output_dir: PathBuf,
    pub builtin_templates: bool,
}

const BUILTIN_STYLESHEET: &str = "style.css";

// pub fn build(options: &BuildOptions) -> Result<BuildReport, BuildErrors> {
//     let root = options.root.as_path();
//     if !root.is_dir() {
//         return Err(NyxError::Io {
//             path: root.to_path_buf(),
//             source: std::io::Error::new(
//                 std::io::ErrorKind::NotFound,
//                 "site root is not a directory",
//             ),
//         }
//         .into());
//     }
// }
