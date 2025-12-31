//! FFI bindings to Fortran core library
//! Type mappings:
//! - Rust `i32`        <-> Fortran `integer(c_int32_t)`
//! - Rust `f32`        <-> Fortran `real(c_float)`
//! - Rust `*const i32` <-> Fortran `integer(c_int32_t), intent(in)`
//! - Rust `*mut f32`   <-> Fortran `real(c_float), intent(out)`

use std::ffi::{c_float, c_int};

/// Status codes returned by Fortran functions
pub mod status {
    pub const SUCCESS: i32 = 0;
    pub const ERR_NULL_POINTER: i32 = -1;
    pub const ERR_INVALID_SIZE: i32 = -2;
    pub const ERR_OUT_OF_MEMORY: i32 = -3;
}

/// Error type for FFI operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiError {
    NullPointer,
    InvalidSize,
    OutOfMemory,
    Unknown(i32),
}

impl FfiError {
    fn from_status(code: i32) -> Option<Self> {
        match code {
            status::SUCCESS => None,
            status::ERR_NULL_POINTER => Some(Self::NullPointer),
            status::ERR_INVALID_SIZE => Some(Self::InvalidSize),
            status::ERR_OUT_OF_MEMORY => Some(Self::OutOfMemory),
            _ => Some(Self::Unknown(code)),
        }
    }
}

unsafe extern "C" {
    /// Smoke test: adds two integers (implemented in Fortran)
    pub fn wvec_add_smoke_test(a: c_int, b: c_int) -> c_int;

    /// Computes sum of a float array (FFI array passing test)
    pub fn wvec_array_sum(arr: *const c_float, n: c_int) -> c_float;

    /// Scales input array into output array: out[i] = in[i] * scale
    /// Returns status code (0 = success)
    pub fn wvec_array_scale(
        arr_in: *const c_float,
        arr_out: *mut c_float,
        n: c_int,
        scale: c_float,
    ) -> c_int;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wvec_add() {
        let result = unsafe { wvec_add_smoke_test(2, 3) };
        assert_eq!(result, 5);
    }

    #[test]
    fn test_array_sum() {
        let arr: [f32; 5] = [1.0, 2.0, 3.0, 4.0, 5.0];
        let result = unsafe { wvec_array_sum(arr.as_ptr(), arr.len() as c_int) };
        assert!((result - 15.0).abs() < 1e-6);
    }

    #[test]
    fn test_array_scale() {
        let input: [f32; 4] = [1.0, 2.0, 3.0, 4.0];
        let mut output: [f32; 4] = [0.0; 4];

        let status = unsafe {
            wvec_array_scale(
                input.as_ptr(),
                output.as_mut_ptr(),
                input.len() as c_int,
                2.5,
            )
        };

        assert_eq!(status, status::SUCCESS);
        assert!((output[0] - 2.5).abs() < 1e-6);
        assert!((output[1] - 5.0).abs() < 1e-6);
        assert!((output[2] - 7.5).abs() < 1e-6);
        assert!((output[3] - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_array_scale_invalid_size() {
        let input: [f32; 1] = [1.0];
        let mut output: [f32; 1] = [0.0];

        let status = unsafe { wvec_array_scale(input.as_ptr(), output.as_mut_ptr(), 0, 1.0) };

        assert_eq!(status, status::ERR_INVALID_SIZE);
    }
}
