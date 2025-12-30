//! FFI bindings to Fortran core library

use std::ffi::c_int;

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
