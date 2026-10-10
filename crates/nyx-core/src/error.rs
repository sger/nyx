use std::{ops::Range, path::PathBuf, sync::Arc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: PathBuf,
    pub text: Arc<str>,
}

impl SourceFile {
    pub fn new(path: impl Into<PathBuf>, text: impl Into<Arc<str>>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self::new(range.start, range.end)
    }
}

#[derive(Debug)]
pub struct TemplateError {
    pub name: String,
    pub file: Option<SourceFile>,
    pub span: Option<Span>,
    pub line: Option<usize>,
    pub message: String,
    pub rendering: Option<String>,
    pub help: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum NyxError {
    #[error("failed to access `{}`", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid site config: {message}")]
    Config {
        file: SourceFile,
        span: Option<Span>,
        message: String,
        help: Option<String>,
    },

    #[error("invalid front matter: {message}")]
    FrontMatter {
        file: SourceFile,
        span: Span,
        message: String,
        help: Option<String>,
    },

    #[error("cannot derive a URL for `{}`: {message}", path.display())]
    InvalidSlug {
        path: PathBuf,
        message: String,
        help: String,
    },

    #[error("template error in `{}`: {}", .0.name, .0.message)]
    Template(Box<TemplateError>),

    #[error("missing template `{}`", path.display())]
    MissingTemplate { path: PathBuf, reason: String },

    #[error("refusing to use `{}` as the output directory: {reason}", path.display())]
    UnsafeOutputDir {
        path: PathBuf,
        reason: String,
        help: String,
    },

    #[error("`{}` would overwrite the rendered `{}`", static_file.display(), path.display())]
    OutputConflict { path: PathBuf, static_file: PathBuf },

    #[error("`{}` is not empty", path.display())]
    DirectoryNotEmpty { path: PathBuf },

    #[error("duplicate permalink `{permalink}`")]
    DuplicatePermalink {
        permalink: String,
        first: PathBuf,
        second: PathBuf,
    },
}

#[derive(Debug, thiserror::Error)]
#[error("build failed with {} error{}", .0.len(), if .0.len() == 1 { "" } else { "s" })]
pub struct BuildErrors(pub Vec<NyxError>);

impl BuildErrors {
    pub fn iter(&self) -> std::slice::Iter<'_, NyxError> {
        self.0.iter()
    }
}

impl From<NyxError> for BuildErrors {
    fn from(error: NyxError) -> Self {
        Self(vec![error])
    }
}

impl IntoIterator for BuildErrors {
    type Item = NyxError;
    type IntoIter = std::vec::IntoIter<NyxError>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a BuildErrors {
    type Item = &'a NyxError;
    type IntoIter = std::slice::Iter<'a, NyxError>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
