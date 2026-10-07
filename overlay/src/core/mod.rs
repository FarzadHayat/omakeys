pub mod config;
pub mod overrides;
pub mod state;

pub use config::{
    load_config, normalize_scale, save_config, Anchor, AppConfig, VisibilityMode,
};
pub use crate::board::labels::LabelStyle;
// AppConfig re-exported for binary.
pub use overrides::{load_overrides, Overrides};
pub use state::{Frame, KeyView, OverlayCore};
