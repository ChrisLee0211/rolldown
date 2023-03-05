use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct TsConfig {
  pub use_define_for_class_fields: bool,
}

impl Default for TsConfig {
  fn default() -> Self {
    Self {
      use_define_for_class_fields: false,
    }
  }
}