use super::LogFragment;
use rltk::prelude::*;
use std::sync::Mutex;

pub const LOGHEIGHT: usize = 12;

lazy_static! {
    static ref LOG: Mutex<Vec<Vec<LogFragment>>> = Mutex::new(Vec::new());
}

#[allow(dead_code)]
pub fn append_fragment(fragment: LogFragment) {
    LOG.lock().unwrap().push(vec![fragment]);
}

pub fn append_entry(fragments: Vec<LogFragment>) {
    LOG.lock().unwrap().push(fragments);
}

pub fn clear_log() {
    LOG.lock().unwrap().clear();
}

pub fn log_display() -> TextBuilder {
    let mut buf = TextBuilder::empty();

    LOG.lock()
        .unwrap()
        .iter()
        .rev()
        .take(LOGHEIGHT)
        .rev()
        .for_each(|log| {
            log.iter().for_each(|frag| {
                buf.fg(frag.color);
                buf.line_wrap(&frag.text);
            });
            buf.ln();
        });

    buf
}

#[allow(dead_code)]
pub fn clone_log() -> Vec<Vec<crate::gamelog::LogFragment>> {
    LOG.lock().unwrap().clone()
}

pub fn restore_log(log: &mut Vec<Vec<crate::gamelog::LogFragment>>) {
    LOG.lock().unwrap().clear();
    LOG.lock().unwrap().append(log);
}
