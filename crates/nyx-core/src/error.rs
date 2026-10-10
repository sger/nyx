//! Errors produced while loading and building a site.
//!
//! A single problem is a [`NyxError`]. A build keeps going after most errors so
//! it can report all of them at once, and returns them together as
//! [`BuildErrors`].
//!
//! Errors caused by the contents of a file carry the [`SourceFile`] and a
//! [`Span`], so the problem can be shown with the offending text highlighted.

use std::{ops::Range, path::PathBuf, sync::Arc};

/// A file's path and full text, kept so an error can show where it happened.
///
/// The text is reference-counted, so cloning a `SourceFile` is cheap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// Path shown in error messages, usually relative to the site root.
    pub path: PathBuf,
    /// The file's contents. [`Span`]s index into this text.
    pub text: Arc<str>,
}

impl SourceFile {
    /// Creates a `SourceFile` from a path and its contents.
    pub fn new(path: impl Into<PathBuf>, text: impl Into<Arc<str>>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
        }
    }
}

/// A range of bytes in a [`SourceFile`]'s text, from `start` up to but not including `end`.
///
/// # Examples
///
/// ```
/// use nyx_core::Span;
///
/// let text = "title = 42";
/// let span = Span::from(8..10);
/// assert_eq!(&text[span.start..span.end], "42");
/// assert_eq!(span.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    /// Byte offset of the first byte.
    pub start: usize,
    /// Byte offset just past the last byte.
    pub end: usize,
}

impl Span {
    /// Creates a span covering `start..end`.
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Number of bytes covered. Returns 0 if `end` is before `start`.
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Returns `true` if the span covers no bytes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl From<Range<usize>> for Span {
    fn from(range: Range<usize>) -> Self {
        Self::new(range.start, range.end)
    }
}

/// Details of a failure while loading or rendering a template.
///
/// Kept separate from [`NyxError`] and boxed in [`NyxError::Template`] so
/// that `NyxError` stays small.
#[derive(Debug)]
pub struct TemplateError {
    /// Name of the template that failed, such as `post.html`.
    pub name: String,
    /// The template's source, if available.
    pub file: Option<SourceFile>,
    /// Location of the problem within `file`.
    pub span: Option<Span>,
    /// Line number of the problem, used when there is no `span`.
    pub line: Option<usize>,
    /// What went wrong.
    pub message: String,
    /// The page being rendered when the error happened.
    pub rendering: Option<String>,
    /// A suggestion for fixing the problem.
    pub help: Option<String>,
}

/// A single problem found while loading or building a site.
///
/// The `Display` text is a one-line summary. Fields such as `help`, `span` and
/// `file` carry the extra detail for a full report.
#[derive(Debug, thiserror::Error)]
pub enum NyxError {
    /// Reading or writing `path` failed.
    #[error("failed to access `{}`", path.display())]
    Io {
        /// The file or directory being accessed.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// `nyx.toml` is not valid TOML or doesn't match the expected settings.
    #[error("invalid site config: {message}")]
    Config {
        /// The config file.
        file: SourceFile,
        /// Location of the bad key or value, if known.
        span: Option<Span>,
        /// What went wrong.
        message: String,
        /// A suggestion for fixing the problem.
        help: Option<String>,
    },

    /// The front matter at the top of a content file is invalid.
    #[error("invalid front matter: {message}")]
    FrontMatter {
        /// The content file.
        file: SourceFile,
        /// Location of the problem.
        span: Span,
        /// What went wrong.
        message: String,
        /// A suggestion for fixing the problem.
        help: Option<String>,
    },

    /// A content file's path can't be turned into a URL.
    #[error("cannot derive a URL for `{}`: {message}", path.display())]
    InvalidSlug {
        /// The content file.
        path: PathBuf,
        /// What went wrong.
        message: String,
        /// A suggestion for fixing the problem.
        help: String,
    },

    /// A template failed to load or render.
    #[error("template error in `{}`: {}", .0.name, .0.message)]
    Template(Box<TemplateError>),

    /// A template the site needs doesn't exist.
    #[error("missing template `{}`", path.display())]
    MissingTemplate {
        /// Where the template was expected.
        path: PathBuf,
        /// Why the template is needed.
        reason: String,
    },

    /// The output directory is somewhere a build must not write, because the
    /// build would delete or overwrite files that aren't its own.
    #[error("refusing to use `{}` as the output directory: {reason}", path.display())]
    UnsafeOutputDir {
        /// The output directory.
        path: PathBuf,
        /// Why the directory is unsafe.
        reason: String,
        /// A suggestion for fixing the problem.
        help: String,
    },

    /// A static file and a rendered page would be written to the same output path.
    #[error("`{}` would overwrite the rendered `{}`", static_file.display(), path.display())]
    OutputConflict {
        /// The rendered page's output path.
        path: PathBuf,
        /// The static file that would replace it.
        static_file: PathBuf,
    },

    /// A directory that must be empty already has files in it.
    #[error("`{}` is not empty", path.display())]
    DirectoryNotEmpty {
        /// The directory.
        path: PathBuf,
    },

    /// Two content files would be published at the same URL.
    #[error("duplicate permalink `{permalink}`")]
    DuplicatePermalink {
        /// The URL both files map to.
        permalink: String,
        /// The first file found with this URL.
        first: PathBuf,
        /// The second file found with this URL.
        second: PathBuf,
    },
}

/// Every error from a failed build. Should contain at least one error.
///
/// A single [`NyxError`] converts into `BuildErrors`, so `?` works in
/// functions that return `Result<_, BuildErrors>`.
///
/// # Examples
///
/// ```
/// use nyx_core::{BuildErrors, NyxError};
///
/// let errors = BuildErrors::from(NyxError::DirectoryNotEmpty { path: "public".into() });
/// assert_eq!(errors.to_string(), "build failed with 1 error");
///
/// for error in &errors {
///     assert_eq!(error.to_string(), "`public` is not empty");
/// }
/// ```
#[derive(Debug, thiserror::Error)]
#[error("build failed with {} error{}", .0.len(), if .0.len() == 1 { "" } else { "s" })]
pub struct BuildErrors(pub Vec<NyxError>);

impl BuildErrors {
    /// Iterates over the errors in the order they were found.
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
