use std::{path::PathBuf, sync::Arc};

use futures::FutureExt;
use rolldown_core::{
  file_name::FileNameTemplate, Bundler, InputItem, InputOptions, InternalModuleFormat,
  OutputOptions,
};
use sugar_path::SugarPathBuf;

// The example try to build esm/cjs output generated by tsc of `@rolldown/core`.

#[tokio::main]
async fn main() {
  // First, you need to figure out how to build esm output `@rolldown/core`.
  let root = PathBuf::from(&std::env::var("CARGO_MANIFEST_DIR").unwrap());
  let cwd = root.join("../../packages/core").into_normalize();

  // build esm
  let mut bundler = Bundler::with_plugins(
    InputOptions {
      input: vec![InputItem {
        name: "index".to_string(),
        import: root
          .join("../../packages/core/src/index.ts")
          .to_string_lossy()
          .to_string(),
      }],
      cwd: cwd.clone(),
      is_external: Arc::new(|specifier, _, _| {
        let res = Ok(specifier == "@rolldown/node-binding");
        async { res }.boxed()
      }),
      ..Default::default()
    },
    vec![rolldown_plugin_node_resolve::NodeResolvePlugin::new_boxed(
      rolldown_plugin_node_resolve::ResolverOptions {
        extensions: vec![
          ".ts".to_string(),
          ".tsx".to_string(),
          ".js".to_string(),
          ".jsx".to_string(),
        ],
        ..Default::default()
      },
      cwd.clone(),
    )],
  );

  bundler
    .write(OutputOptions {
      // FIXME: If we don't overwrite the filenames, the filenames would be `src_index.mjs`
      entry_file_names: FileNameTemplate::new("../lib/index.mjs".to_string()),
      format: InternalModuleFormat::Esm,
      ..Default::default()
    })
    .await
    .unwrap();
  // build cjs

  // build esm
  let mut bundler = Bundler::with_plugins(
    InputOptions {
      input: vec![InputItem {
        name: "index".to_string(),
        import: root
          .join("../../packages/core/src/index.ts")
          .to_string_lossy()
          .to_string(),
      }],
      cwd: cwd.clone(),
      is_external: Arc::new(|specifier, _, _| {
        let res = Ok(specifier == "@rolldown/node-binding");
        async { res }.boxed()
      }),
      ..Default::default()
    },
    vec![rolldown_plugin_node_resolve::NodeResolvePlugin::new_boxed(
      rolldown_plugin_node_resolve::ResolverOptions {
        extensions: vec![
          ".ts".to_string(),
          ".tsx".to_string(),
          ".js".to_string(),
          ".jsx".to_string(),
        ],
        ..Default::default()
      },
      cwd.clone(),
    )],
  );

  bundler
    .write(OutputOptions {
      // FIXME: If we don't overwrite the filenames, the filenames would be `src_index.js`
      entry_file_names: FileNameTemplate::new("index.js".to_string()),
      format: InternalModuleFormat::Cjs,
      ..Default::default()
    })
    .await
    .unwrap();
}