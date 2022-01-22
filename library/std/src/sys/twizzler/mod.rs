use crate::intrinsics;
use crate::os::raw::c_char;

pub mod alloc;
pub mod args;
#[path = "../unix/cmath.rs"]
pub mod cmath;
#[path = "../unsupported/condvar.rs"]
pub mod condvar;
pub mod env;
#[path = "../unsupported/fs.rs"]
pub mod fs;
#[path = "../unsupported/io.rs"]
pub mod io;
pub mod memchr;
#[path = "../unsupported/mutex.rs"]
pub mod mutex;
#[path = "../unsupported/net.rs"]
pub mod net;
pub mod os;
#[path = "../unix/os_str.rs"]
pub mod os_str;
#[path = "../unix/path.rs"]
pub mod path;
#[path = "../unsupported/pipe.rs"]
pub mod pipe;
#[path = "../unsupported/process.rs"]
pub mod process;
#[path = "../unsupported/rwlock.rs"]
pub mod rwlock;
#[path = "../unsupported/stdio.rs"]
pub mod stdio;
#[path = "../unsupported/thread.rs"]
pub mod thread;
pub mod thread_local_dtor;
#[path = "../unsupported/thread_local_key.rs"]
pub mod thread_local_key;
#[path = "../unsupported/time.rs"]
pub mod time;

pub fn unsupported<T>() -> crate::io::Result<T> {
    Err(unsupported_err())
}

pub fn unsupported_err() -> crate::io::Error {
    crate::io::Error::new_const(
        crate::io::ErrorKind::Unsupported,
        &"operation not supported on Twizzler yet",
    )
}

pub unsafe fn strlen(start: *const c_char) -> usize {
    let mut str = start;

    while *str != 0 {
        str = str.offset(1);
    }

    (str as usize) - (start as usize)
}

#[no_mangle]
pub extern "C" fn floor(x: f64) -> f64 {
    unsafe { intrinsics::floorf64(x) }
}

pub fn abort_internal() -> ! {
    loop {} //TODO
}

// This function is needed by the panic runtime. The symbol is named in
// pre-link args for the target specification, so keep that in sync.
#[cfg(not(test))]
#[no_mangle]
// NB. used by both libunwind and libpanic_abort
pub extern "C" fn __rust_abort() {
    abort_internal();
}

// SAFETY: must be called only once during runtime initialization.
// NOTE: this is not guaranteed to run, for example when Rust code is called externally.
pub unsafe fn init(argc: isize, argv: *const *const u8) {
    args::init(argc, argv);
}

// SAFETY: must be called only once during runtime cleanup.
// NOTE: this is not guaranteed to run, for example when the program aborts.
pub unsafe fn cleanup() {
    args::cleanup();
}

pub fn decode_error_kind(_errno: i32) -> crate::io::ErrorKind {
    unimplemented!()
}

// FIXME: just a workaround to test the system
pub fn hashmap_random_keys() -> (u64, u64) {
    (1, 2)
}

#[no_mangle]
#[allow(unreachable_code)]
#[allow(unused_variables)]
pub unsafe extern "C" fn std_runtime_start(env: *const *const i8) {
    extern "C" {
        fn main(argc: isize, argv: *const *const c_char) -> i32;
    }
    //crate::sys::os::init_environment(env);
    twizzler_abi::ready();
    main(0, core::ptr::null());
    thread_local_dtor::run_dtors();
    __rust_abort();
}
