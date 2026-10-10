use std::{
    io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::toml_diag;
use crate::{NyxError, SourceFile};

pub const CONFIG_FILE: &str = "nyx.toml";
const KNOWN_FIELDS: &[&str] = &["title", "base_url", "description", "author", "output_dir"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SiteConfig {
    pub title: String,
    pub base_url: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            title: "Nyx site".to_owned(),
            base_url: "/".to_owned(),
            description: None,
            author: None,
            output_dir: default_output_dir(),
        }
    }
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("public")
}

pub fn load_config(root: &Path) -> Result<SiteConfig, NyxError> {
    let path = root.join(CONFIG_FILE);
    match std::fs::read_to_string(&path) {
        Ok(text) => parse_config(SourceFile::new(CONFIG_FILE, text)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(SiteConfig::default()),
        Err(source) => Err(NyxError::Io { path, source }),
    }
}

pub fn parse_config(file: SourceFile) -> Result<SiteConfig, NyxError> {
    toml::from_str(&file.text).map_err(|err| {
        let message = err.message().to_owned();
        NyxError::Config {
            span: err.span().map(Into::into),
            help: toml_diag::help_for(&message, CONFIG_FILE, KNOWN_FIELDS),
            message,
            file,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Span;

    fn parse(text: &str) -> Result<SiteConfig, NyxError> {
        parse_config(SourceFile::new(CONFIG_FILE, text))
    }

    fn config_error(text: &str) -> (Option<Span>, String, Option<String>) {
        match parse(text) {
            Err(NyxError::Config {
                span,
                message,
                help,
                ..
            }) => (span, message, help),
            other => panic!("expected config error, got {other:?}"),
        }
    }

    #[test]
    fn parses_all_fields() {
        let config = parse(
            r#"
title = "Spiros' Notes"
base_url = "https://example.com"
description = "Rust, iOS and tooling"
author = "Spiros"
output_dir = "dist"
"#,
        )
        .unwrap();

        assert_eq!(
            config,
            SiteConfig {
                title: "Spiros' Notes".to_owned(),
                base_url: "https://example.com".to_owned(),
                description: Some("Rust, iOS and tooling".to_owned()),
                author: Some("Spiros".to_owned()),
                output_dir: PathBuf::from("dist"),
            }
        );
    }

    #[test]
    fn optional_fields_default() {
        let config = parse("title = \"T\"\nbase_url = \"https://example.com\"\n").unwrap();
        assert_eq!(config.description, None);
        assert_eq!(config.author, None);
        assert_eq!(config.output_dir, PathBuf::from("public"));
    }

    #[test]
    fn missing_required_field_is_an_error_with_help() {
        let (_, message, help) = config_error("base_url = \"https://example.com\"\n");
        assert!(message.contains("missing field `title`"), "{message}");
        assert_eq!(help.as_deref(), Some("add `title = \"...\"` to nyx.toml"));
    }

    #[test]
    fn unknown_field_points_at_the_key() {
        let text = "title = \"T\"\nbase_url = \"https://example.com\"\ntitel = \"oops\"\n";
        let (span, message, help) = config_error(text);
        assert!(message.contains("unknown field `titel`"), "{message}");
        let span = span.unwrap();
        assert_eq!(&text[span.start..span.end], "titel");
        assert!(help.unwrap().contains("title, base_url"));
    }

    #[test]
    fn wrong_type_points_at_the_value() {
        let text = "title = 42\nbase_url = \"https://example.com\"\n";
        let (span, _, _) = config_error(text);
        let span = span.unwrap();
        assert_eq!(&text[span.start..span.end], "42");
    }

    #[test]
    fn missing_file_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load_config(dir.path()).unwrap(), SiteConfig::default());
    }

    #[test]
    fn loads_file_from_root() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(CONFIG_FILE),
            "title = \"T\"\nbase_url = \"https://example.com\"\n",
        )
        .unwrap();
        assert_eq!(load_config(dir.path()).unwrap().title, "T");
    }

    #[test]
    fn invalid_file_reports_root_relative_path() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(CONFIG_FILE), "title = \n").unwrap();
        match load_config(dir.path()) {
            Err(NyxError::Config { file, .. }) => assert_eq!(file.path, Path::new(CONFIG_FILE)),
            other => panic!("expected config error, got {other:?}"),
        }
    }
}
