use std::{
  borrow::Cow,
  path::{Path, PathBuf},
};

use nodejs_resolver::{Options, Resolver as EnhancedResolver};
use sugar_path::AsPath;

#[derive(Debug)]
pub struct Resolver {
  cwd: PathBuf,
  inner: EnhancedResolver,
}

impl Resolver {
  pub fn with_cwd(cwd: PathBuf, preserve_symlinks: bool) -> Self {
    Self {
      cwd,
      inner: EnhancedResolver::new(Options {
        symlinks: !preserve_symlinks,
        extensions: vec![
          ".js".to_string(),
          ".jsx".to_string(),
          ".ts".to_string(),
          ".tsx".to_string(),
        ],
        // TODO(hyf0): Should we set this as default?
        prefer_relative: false,
        ..Default::default()
      }),
    }
  }

  pub fn cwd(&self) -> &PathBuf {
    &self.cwd
  }
}

impl Default for Resolver {
  fn default() -> Self {
    Self::with_cwd(std::env::current_dir().unwrap(), true)
  }
}

impl Resolver {
  pub fn resolve(&self, importer: Option<&str>, specifier: &str) -> rolldown_error::Result<String> {
    // If the importer is `None`, it means that the specifier is the entry file.
    // In this case, we could not simple use the CWD as the importer.
    // Instead, we should concat the CWD with the specifier. This aligns with https://github.com/rollup/rollup/blob/680912e2ceb42c8d5e571e01c6ece0e4889aecbb/src/utils/resolveId.ts#L56.
    let specifier = if importer.is_none() {
      Cow::Owned(self.cwd.join(specifier))
    } else {
      Cow::Borrowed(Path::new(specifier))
    };

    let importer_dir = importer
      .map(|s| Path::new(s).parent().expect("Should have a parent dir"))
      .unwrap_or(&self.cwd);

    let resolved = self
      .inner
      .resolve(importer_dir, &specifier.to_string_lossy());
    match resolved {
      Ok(resolved) => match resolved {
        nodejs_resolver::ResolveResult::Info(info) => Ok(info.path().to_string_lossy().to_string()),
        nodejs_resolver::ResolveResult::Ignored => unreachable!(),
      },
      Err(_err) => {
        if let Some(importer) = importer {
          Err(rolldown_error::Error::unresolved_import(
            specifier.to_string_lossy().to_string(),
            importer.as_path().to_path_buf(),
          ))
        } else {
          Err(rolldown_error::Error::unresolved_entry(specifier))
        }
      }
    }
  }
}
