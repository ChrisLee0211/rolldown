use rayon::prelude::*;
use rolldown_common::Symbol;
use rolldown_error::Errors;
use rustc_hash::FxHashSet;
use swc_core::common::GLOBALS;
use tracing::instrument;

use super::TreeshakeContext;
use crate::{treeshake::TreeshakeNormalModule, BuildResult, Graph, COMPILER, SWC_GLOBALS};

impl Graph {
  #[instrument(skip_all)]
  pub(crate) fn treeshake(&mut self) -> BuildResult<()> {
    let used_ids = self
      .collect_all_used_ids()?
      .into_iter()
      .map(|id| id.to_id())
      .collect();

    self
      .module_by_id
      .values_mut()
      .par_bridge()
      .filter_map(|m| m.as_norm_mut())
      .for_each(|module| {
        GLOBALS.set(&SWC_GLOBALS, || {
          tracing::trace!(
            "[before treeshake]module: {},code: \n{}",
            module.id,
            COMPILER.debug_print(&module.ast, None).unwrap()
          );

          rolldown_swc_visitors::treeshake(
            &mut module.ast,
            self.unresolved_mark,
            &used_ids,
            module.top_level_ctxt,
            GLOBALS.set(&SWC_GLOBALS, || module.top_level_ctxt.outer()),
            COMPILER.cm.clone(),
            &module.comments,
          );
          tracing::trace!(
            "[after treeshake]module: {},code: \n{}",
            module.id,
            COMPILER.debug_print(&module.ast, None).unwrap()
          );
          // We don't need `export`, because of scope hoisting.
          rolldown_swc_visitors::remove_export_and_import(&mut module.ast);
        });

        module
          .linked_imports
          .values_mut()
          .par_bridge()
          .for_each(|specs| specs.retain(|spec| used_ids.contains(spec.imported_as.as_id())));

        module
          .linked_exports
          .retain(|_exported_name, spec| used_ids.contains(spec.local_id.as_id()));

        module.parts.parts.iter_mut().for_each(|part| {
          // If the Symbol is unused, delete it.
          // So, in deconflicting, it won't take up a name meaninglessly.
          part
            .declared
            .retain(|symbol| used_ids.contains(symbol.as_id()));
        });
      });
    Ok(())
  }

  #[instrument(skip_all)]
  pub(crate) fn collect_all_used_ids(&mut self) -> BuildResult<FxHashSet<Symbol>> {
    let ctx = TreeshakeContext {
      id_to_module: self
        .module_by_id
        .par_iter()
        .filter_map(|(id, m)| m.as_norm().map(|m| (id, TreeshakeNormalModule::new(m))))
        .collect(),
      errors: Default::default(),
    };
    let used_ids = ctx
      .id_to_module
      .values()
      .par_bridge()
      .map(|m| m.include(&ctx))
      .flatten()
      .collect::<FxHashSet<_>>();
    let errors = ctx.errors.into_inner().unwrap();
    if !errors.is_empty() {
      return Err(Errors::from_vec(errors));
    }
    Ok(used_ids)
  }
}
