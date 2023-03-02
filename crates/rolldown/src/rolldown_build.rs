use std::path::PathBuf;

use rolldown_core::{Asset, BuildResult, BundlerCore};
use rolldown_plugin::BuildPlugin;
use sugar_path::AsPath;

use crate::InputOptions;

pub struct Bundler {
  core: BundlerCore,
  cwd: PathBuf,
}

impl Bundler {
  pub fn new(input_opts: InputOptions) -> Self {
    Self::with_plugins(input_opts, vec![])
  }

  pub fn with_plugins(input_opts: InputOptions, mut plugins: Vec<Box<dyn BuildPlugin>>) -> Self {
    rolldown_tracing::enable_tracing_on_demand();
    let cwd = input_opts.cwd.clone();

    let mut builtin_post_plugins = vec![];

    if let Some(node_resolve) = input_opts.builtins.node_resolve {
      builtin_post_plugins.push(rolldown_plugin_node_resolve::NodeResolvePlugin::new_boxed(
        rolldown_plugin_node_resolve::ResolverOptions {
          extensions: node_resolve.extensions,
          symlinks: !input_opts.preserve_symlinks,
          ..Default::default()
        },
        cwd.clone(),
      ))
    }

    plugins.extend(builtin_post_plugins);

    let bundler = BundlerCore::with_plugins(
      rolldown_core::BuildInputOptions {
        input: input_opts.input,
        treeshake: input_opts.treeshake,
        cwd: input_opts.cwd,
        is_external: input_opts.is_external,
        on_warn: input_opts.on_warn,
      },
      plugins,
    );
    Self { cwd, core: bundler }
  }

  pub async fn write(&mut self, output_options: crate::OutputOptions) -> BuildResult<Vec<Asset>> {
    let dir = output_options.dir.clone().unwrap_or_else(|| {
      self
        .cwd
        .as_path()
        .join("dist")
        .to_string_lossy()
        .to_string()
    });
    let output = self
      .core
      .build(rolldown_core::BuildOutputOptions {
        entry_file_names: output_options.entry_file_names,
        chunk_file_names: output_options.chunk_file_names,
        format: output_options.format,
        export_mode: output_options.export_mode,
      })
      .await?;

    std::fs::create_dir_all(&dir).unwrap_or_else(|_| {
      panic!(
        "Could not create directory for output chunks: {:?} \ncwd: {}",
        dir.as_path(),
        self.cwd.display()
      )
    });
    for chunk in &output {
      let dest = dir.as_path().join(&chunk.filename);
      if let Some(p) = dest.parent() {
        if !p.exists() {
          std::fs::create_dir_all(p)?;
        }
      };
      std::fs::write(dest, &chunk.content).unwrap_or_else(|_| {
        panic!(
          "Failed to write file in {:?}",
          dir.as_path().join(&chunk.filename)
        )
      });
    }
    Ok(output)
  }

  pub async fn generate(
    &mut self,
    output_options: crate::OutputOptions,
  ) -> BuildResult<Vec<Asset>> {
    let output = self
      .core
      .build(rolldown_core::BuildOutputOptions {
        entry_file_names: output_options.entry_file_names,
        chunk_file_names: output_options.chunk_file_names,
        format: output_options.format,
        export_mode: output_options.export_mode,
      })
      .await?;

    Ok(output)
  }
}
