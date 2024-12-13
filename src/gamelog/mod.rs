use rltk::RGB;
mod logstore;
use logstore::*;
pub use logstore::{clear_log, log_display, LOGHEIGHT};
mod builder;
pub use builder::*;

pub struct LogFragment {
    pub color: RGB,
    pub text: String,
}
