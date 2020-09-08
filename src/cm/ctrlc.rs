static mut CTRLC: bool = false;

extern fn callback(_a: i32) {
    unsafe { CTRLC = true; }
}

pub fn init() {
    unsafe {
        libc::signal(libc::SIGINT, callback as libc::sighandler_t);
    }
}

pub fn poll() -> bool {
    unsafe {
        let result = CTRLC;
        if CTRLC {
            CTRLC = false;
        }
        result
    }
}
