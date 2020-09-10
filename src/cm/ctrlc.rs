use std::sync::atomic::{AtomicBool, Ordering};

static CTRLC: AtomicBool = AtomicBool::new(false);

// TODO(#181): ctrlc module is not implemented for windows

#[cfg(unix)]
extern "C" fn callback(_signum: i32) {
    CTRLC.store(true, Ordering::Relaxed);
}

pub fn init() {
    if cfg!(unix) {
        unsafe {
            // TODO(#182): Explore portability issues of using signal(2)
            libc::signal(libc::SIGINT, callback as libc::sighandler_t);
        }
    }
}

pub fn poll() -> bool {
    CTRLC.swap(false, Ordering::Relaxed)
}
