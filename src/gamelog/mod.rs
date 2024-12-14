use rltk::RGB;
mod logstore;
use logstore::*;
pub use logstore::{clear_log, clone_log, log_display, restore_log, LOGHEIGHT};
mod builder;
pub use builder::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct LogFragment {
    pub color: RGB,
    pub text: String,
}
