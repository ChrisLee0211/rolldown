use std::path::PathBuf;

use derivative::Derivative;
use futures::future::join_all;
use rolldown_common::{Loader, ModuleId};
use rolldown_error::Errors;
use rolldown_resolver::Resolver;
use rolldown_swc_visitors::ScanResult;
use rustc_hash::FxHashMap;
use sugar_path::AsPath;
use swc_core::common::{Mark, SyntaxContext, GLOBALS};
use swc_core::ecma::ast;
use swc_core::ecma::atoms::JsWord;
use swc_core::ecma::parser::{EsConfig, Syntax, TsConfig};
use swc_node_comments::SwcComments;
use tracing::instrument;

use super::Msg;
use crate::{
  extract_loader_by_path, resolve_id, BuildError, BuildResult, IsExternal, ResolvedModuleIds,
  SharedBuildInputOptions, SharedBuildPluginDriver, SharedResolver, UnaryBuildResult, COMPILER,
  SWC_GLOBALS,
};

pub(crate) struct ModuleTask {
  pub(crate) input_options: SharedBuildInputOptions,
  pub(crate) id: ModuleId,
  pub(crate) is_user_defined_entry: bool,
  pub(crate) tx: tokio::sync::mpsc::UnboundedSender<Msg>,
  pub(crate) top_level_mark: Mark,
  pub(crate) top_level_ctxt: SyntaxContext,
  pub(crate) unresolved_mark: Mark,
  pub(crate) unresolved_ctxt: SyntaxContext,
  pub(crate) resolver: SharedResolver,
  pub(crate) plugin_driver: SharedBuildPluginDriver,
  pub(crate) is_external: IsExternal,
}

impl ModuleTask {
  // I(hyf0) have no interest in implementing original resolve logic of rollup currently.
  // It's complicated and I doubt the usage of it.
  pub(crate) async fn resolve_id(
    resolver: &Resolver,
    importer: &ModuleId,
    specifier: &str,
    plugin_driver: &SharedBuildPluginDriver,
    is_external: &IsExternal,
  ) -> UnaryBuildResult<ModuleId> {
    let is_marked_as_external = is_external(specifier, Some(importer.id()), false).await?;

    if is_marked_as_external {
      return Ok(ModuleId::new(specifier, true));
    }

    let resolved_id = resolve_id(resolver, specifier, Some(importer), false, plugin_driver).await?;

    if let Some(resolved) = resolved_id {
      let is_resolved_marked_as_external =
        is_external(resolved.id(), Some(importer.id()), true).await?;

      Ok(ModuleId::new(
        resolved.id().clone(),
        is_resolved_marked_as_external,
      ))
    } else {
      // TODO: emit warnings like https://rollupjs.org/guide/en#warning-treating-module-as-external-dependency
      Ok(ModuleId::new(specifier, true))
    }
  }

  #[instrument(skip_all)]
  pub(crate) async fn run(self) {
    let tx = self.tx.clone();
    match self.run_inner().await {
      Ok(result) => {
        tx.send(Msg::Scanned(result)).unwrap();
      }
      Err(err) => {
        tx.send(Msg::Error(err)).unwrap();
      }
    }
  }

  async fn resolve_dependencies(
    &self,
    result: &ScanResult,
  ) -> BuildResult<FxHashMap<JsWord, ModuleId>> {
    let dependencies = result
      .dependencies
      .iter()
      .chain(result.dyn_dependencies.iter());

    let jobs = dependencies.cloned().map(|specifier| {
      let resolver = self.resolver.clone();
      let plugin_driver = self.plugin_driver.clone();
      let importer = self.id.clone();
      let is_external = self.is_external.clone();

      tokio::spawn(async move {
        Self::resolve_id(
          &resolver,
          &importer,
          &specifier,
          &plugin_driver,
          &is_external,
        )
        .await
        .map(|id| (specifier.clone(), id))
      })
    });

    let resolved_ids = join_all(jobs).await;

    let mut errors = vec![];

    let ret: FxHashMap<JsWord, ModuleId> = resolved_ids
      .into_iter()
      .filter_map(|handle| match handle.unwrap() {
        Ok(id) => Some(id),
        Err(e) => {
          errors.push(e);
          None
        }
      })
      .collect();

    if errors.is_empty() {
      Ok(ret)
    } else {
      Err(Errors::from_vec(errors))
    }
  }

  async fn run_inner(self) -> BuildResult<TaskResult> {
    // load hook
    let code = tokio::fs::read_to_string(self.id.as_ref())
      .await
      .map_err(|e| BuildError::io_error(e))
      .map_err(|e| e.context(format!("Read file: {}", self.id.as_ref())))?;

    let loader = if self.input_options.builtins.detect_loader_by_ext {
      extract_loader_by_path(self.id.as_path())
    } else {
      Loader::Js
    };

    let code = self
      .plugin_driver
      .read()
      .await
      .transform(&self.id, code)
      .await?;

    let (mut ast, comments) = parse_to_js_ast(&self.id, code, loader, &self.input_options)?;

    // No matter what, the ast should be a pure valid JavaScript in this phrase
    GLOBALS.set(&SWC_GLOBALS, || {
      rolldown_swc_visitors::resolve(&mut ast, self.unresolved_mark, self.top_level_mark);
    });

    let result = rolldown_swc_visitors::scan(
      &mut ast,
      self.top_level_ctxt,
      self.unresolved_ctxt,
      self.id.clone(),
    );

    let resolved_ids = self.resolve_dependencies(&result).await?;

    Ok(TaskResult {
      module_id: self.id,
      ast,
      top_level_mark: self.top_level_mark,
      top_level_ctxt: self.top_level_ctxt,
      scan_result: result,
      resolved_ids,
      comments,
      is_user_defined_entry: self.is_user_defined_entry,
    })
  }
}

#[derive(Derivative)]
#[derivative(Debug)]
pub(crate) struct TaskResult {
  pub module_id: ModuleId,
  pub ast: ast::Module,
  pub top_level_mark: Mark,
  pub top_level_ctxt: SyntaxContext,
  pub scan_result: ScanResult,
  pub resolved_ids: ResolvedModuleIds,
  #[derivative(Debug = "ignore")]
  pub comments: SwcComments,
  pub is_user_defined_entry: bool,
}

/// This function should emit valid JavaScript AST(with JSX)
fn parse_to_js_ast(
  id: &ModuleId,
  source: String,
  loader: Loader,
  input_options: &SharedBuildInputOptions,
) -> UnaryBuildResult<(ast::Module, SwcComments)> {
  match loader {
    Loader::Js | Loader::Jsx | Loader::Ts | Loader::Tsx => {
      let is_jsx_or_tsx = matches!(loader, Loader::Jsx | Loader::Tsx);
      let is_ts_or_tsx = matches!(loader, Loader::Ts | Loader::Tsx);
      let syntax = if is_ts_or_tsx {
        Syntax::Typescript(TsConfig {
          tsx: is_jsx_or_tsx,
          decorators: true,
          ..Default::default()
        })
      } else {
        Syntax::Es(EsConfig {
          jsx: is_jsx_or_tsx,
          ..Default::default()
        })
      };
      let comments = SwcComments::default();
      let fm = COMPILER.create_source_file(PathBuf::from(id.as_ref().to_string()), source);
      let mut ast = COMPILER
        .parse_with_comments(fm.clone(), syntax, Some(&comments))
        .map_err(|e| BuildError::parse_js_failed(fm, e).context(format!("{loader:?}")))?;
      if is_ts_or_tsx {
        rolldown_swc_visitors::ts_to_js(
          &mut ast,
          rolldown_swc_visitors::TsConfig {
            use_define_for_class_fields: input_options
              .builtins
              .tsconfig
              .use_define_for_class_fields,
            ..Default::default()
          },
        );
      }
      Ok((ast, comments))
    }
    Loader::Json => unimplemented!(),
  }
}
