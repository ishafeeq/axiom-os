pub use axiom_macros::axiom_api;
pub use axiom_macros::axiom_export_reflect;

pub mod http {
    #[link(wasm_import_module = "axiom")]
    unsafe extern "C" {
        /// Calls the Host's HTTP proxy. 
        /// Returns a pointer to the null-terminated response string in Wasm memory.
        fn http_call(
            alias_ptr: *const u8, 
            method_ptr: *const u8, 
            body_ptr: *const u8, 
            body_len: u32
        ) -> u32;
    }

    pub fn get(alias: &str) -> String {
        call(alias, "GET", None)
    }

    pub fn post(alias: &str, body: &str) -> String {
        call(alias, "POST", Some(body))
    }

    pub fn put(alias: &str, body: &str) -> String {
        call(alias, "PUT", Some(body))
    }

    pub fn delete(alias: &str) -> String {
        call(alias, "DELETE", None)
    }

    fn call(alias: &str, method: &str, body: Option<&str>) -> String {
        let alias_null = format!("{}\0", alias);
        let method_null = format!("{}\0", method);
        
        let (b_ptr, b_len) = match body {
            Some(b) => (b.as_ptr(), b.len() as u32),
            None => (std::ptr::null(), 0),
        };

        let ptr = unsafe { 
            http_call(
                alias_null.as_ptr(), 
                method_null.as_ptr(),
                b_ptr,
                b_len
            ) 
        };
        
        if ptr == 0 {
            return "Error: HTTP call failed".to_string();
        }

        let c_str = unsafe { std::ffi::CStr::from_ptr(ptr as *const i8) };
        c_str.to_string_lossy().into_owned()
    }
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

pub mod db {
    use serde::{Serialize, Deserialize};

    #[link(wasm_import_module = "axiom")]
    unsafe extern "C" {
        /// Calls the Host's database bridge.
        fn db_execute(
            alias_ptr: *const u8,
            query_ptr: *const u8,
            query_len: u32
        ) -> u32;
    }

    #[derive(Serialize, Deserialize)]
    pub struct AxiomQuery {
        pub sql: String,
        pub params: Vec<serde_json::Value>,
    }

    #[derive(Serialize, Deserialize)]
    pub struct AxiomResponse {
        pub rows: Vec<serde_json::Value>,
        pub affected_rows: u64,
    }

    pub struct QueryBuilder {
        alias: String,
        sql: String,
        params: Vec<serde_json::Value>,
    }

    impl QueryBuilder {
        pub fn new(alias: &str) -> Self {
            Self {
                alias: alias.to_string(),
                sql: String::new(),
                params: Vec::new(),
            }
        }

        pub fn query(mut self, sql: &str) -> Self {
            self.sql = sql.to_string();
            self
        }

        pub fn bind<T: Serialize>(mut self, value: T) -> Self {
            if let Ok(json) = serde_json::to_value(value) {
                self.params.push(json);
            }
            self
        }

        pub fn execute(self) -> Result<AxiomResponse, String> {
            let query = AxiomQuery {
                sql: self.sql,
                params: self.params,
            };
            let query_json = serde_json::to_string(&query).map_err(|e| e.to_string())?;
            let alias_null = format!("{}\0", self.alias);

            let ptr = unsafe {
                db_execute(
                    alias_null.as_ptr(),
                    query_json.as_ptr(),
                    query_json.len() as u32
                )
            };

            if ptr == 0 {
                return Err("Database query failed".to_string());
            }

            let c_str = unsafe { std::ffi::CStr::from_ptr(ptr as *const i8) };
            let res_json = c_str.to_string_lossy();
            serde_json::from_str(&res_json).map_err(|e| e.to_string())
        }
    }
}

pub mod health {
    #[link(wasm_import_module = "axiom")]
    unsafe extern "C" {
        /// Checks the health status of a logical binding.
        /// Returns "Closed" (Healthy), "Open" (Blocked), or "HalfOpen".
        fn axiom_health_status(alias_ptr: *const u8) -> u32;
    }

    pub fn get_binding_status(alias: &str) -> String {
        let alias_null = format!("{}\0", alias);
        let ptr = unsafe { axiom_health_status(alias_null.as_ptr()) };
        
        if ptr == 0 {
            return "Unknown".to_string();
        }

        let c_str = unsafe { std::ffi::CStr::from_ptr(ptr as *const i8) };
        c_str.to_string_lossy().into_owned()
    }
}
