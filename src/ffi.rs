//! FFI bindings to Fortran core library
//! Type mappings:
//! - Rust `i32`        <-> Fortran `integer(c_int32_t)`
//! - Rust `f32`        <-> Fortran `real(c_float)`
//! - Rust `*const i32` <-> Fortran `integer(c_int32_t), intent(in)`
//! - Rust `*mut f32`   <-> Fortran `real(c_float), intent(out)`

use std::ffi::c_int;

/// Status codes returned by Fortran functions
pub mod status {
    pub const SUCCESS: i32 = 0;
    pub const ERR_NULL_POINTER: i32 = -1;
    pub const ERR_INVALID_SIZE: i32 = -2;
    pub const ERR_OUT_OF_MEMORY: i32 = -3;
}

unsafe extern "C" {
    /// Smoke test: adds two integers (implemented in Fortran)
    pub fn wvec_add_smoke_test(a: c_int, b: c_int) -> c_int;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wvec_add() {
        let result = unsafe { wvec_add_smoke_test(2, 3) };
        assert_eq!(result, 5);
    }
}
