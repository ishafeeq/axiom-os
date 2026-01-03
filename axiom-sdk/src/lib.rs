pub use axiom_macros::axiom_api;
pub use axiom_macros::axiom_export_reflect;

// Internal trait to help collect metadata (hidden from docs)
#[doc(hidden)]
pub trait AxiomApiMetadata {
    fn metadata() -> &'static str;
}

#[link(wasm_import_module = "axiom")]
unsafe extern "C" {
    pub fn axiom_log(ptr: *const u8, len: usize, level: u32);
}

#[doc(hidden)]
pub fn __axiom_log_internal(msg: &str, level: u32) {
    unsafe {
        axiom_log(msg.as_ptr(), msg.len(), level);
    }
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        $crate::__axiom_log_internal(&format!($($arg)*), 2);
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::__axiom_log_internal(&format!($($arg)*), 1);
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        $crate::__axiom_log_internal(&format!($($arg)*), 0);
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        $crate::__axiom_log_internal(&format!($($arg)*), 3);
    };
}

// Internal trait to help collect metadata (hidden from docs)
#[doc(hidden)]
pub trait AxiomApiMetadata {
    fn metadata() -> &'static str;
}

#[macro_export]
macro_rules! axiom_runtime {
    () => {
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn axiom_init() {
            // SDK Initialization
        }
    };
}
