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
    pub const ERR_FILE_IO: i32 = -4;
    pub const ERR_INVALID_MAGIC: i32 = -5;
    pub const ERR_UNSUPPORTED_VERSION: i32 = -6;
    pub const STATUS_INTERRUPTED: i32 = 1;
}

/// Error type for FFI operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiError {
    NullPointer,
    InvalidSize,
    OutOfMemory,
    Unknown(i32),
    FileIo,
    InvalidMagic,
    UnsupportedVersion,
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

impl std::fmt::Display for FfiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NullPointer => write!(f, "null pointer"),
            Self::InvalidSize => write!(f, "invalid size"),
            Self::OutOfMemory => write!(f, "out of memory"),
            Self::Unknown(code) => write!(f, "unknown error (code {})", code),
            Self::FileIo => write!(f, "file I/O error"),
            Self::InvalidMagic => write!(f, "invalid checkpoint magic bytes"),
            Self::UnsupportedVersion => write!(f, "unsupported checkpoint version"),
        }
    }
}

impl std::error::Error for FfiError {}

/// Safe wrapper: scales an array by a constant factor
pub fn array_scale(input: &[f32], scale: f32) -> Result<Vec<f32>, FfiError> {
    if input.is_empty() {
        return Err(FfiError::InvalidSize);
    }

    let mut output = vec![0.0f32; input.len()];

    let status = unsafe {
        wvec_array_scale(
            input.as_ptr(),
            output.as_mut_ptr(),
            input.len() as c_int,
            scale,
        )
    };

    match FfiError::from_status(status) {
        None => Ok(output),
        Some(err) => Err(err),
    }
}

/// Safe wrapper: computes sum of array elements
pub fn array_sum(arr: &[f32]) -> f32 {
    if arr.is_empty() {
        return 0.0;
    }
    unsafe { wvec_array_sum(arr.as_ptr(), arr.len() as c_int) }
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

    // Word2Vec model functions
    /// Initialize model with vocab_size and embedding dimension
    pub fn wvec_model_init(vocab_size: c_int, dim: c_int) -> c_int;

    /// Free model memory
    pub fn wvec_model_free();

    /// Get model dimensions
    pub fn wvec_model_get_dims(vocab_size: *mut c_int, dim: *mut c_int);

    /// Check if model is initialized (returns 1 if true, 0 if false)
    pub fn wvec_model_is_init() -> c_int;

    /// Copy embedding for word_id to output buffer
    pub fn wvec_get_embedding(word_id: c_int, out_vec: *mut c_float, out_len: c_int) -> c_int;

    /// Train one skip-gram pair with negative sampling
    pub fn wvec_train_pair(
        center_id: c_int,
        context_id: c_int,
        neg_ids: *const c_int,
        n_neg: c_int,
        lr: c_float,
    ) -> c_int;

    /// Train on corpus with OpenMP parallelization
    pub fn wvec_train_corpus(
        token_ids: *const c_int,
        n_tokens: c_int,
        window: c_int,
        n_neg: c_int,
        neg_table: *const c_int,
        neg_table_size: c_int,
        lr: c_float,
    ) -> c_int;

    // Checkpoint functions
    /// Save model state to checkpoint file
    /// Returns 0 on success, negative on error
    pub fn wvec_checkpoint_save(
        filename: *const std::ffi::c_char,
        filename_len: c_int,
        epoch: c_int,
        learning_rate: c_float,
    ) -> c_int;

    /// Load model state from checkpoint file
    /// Returns 0 on success, negative on error
    pub fn wvec_checkpoint_load(
        filename: *const std::ffi::c_char,
        filename_len: c_int,
        epoch: *mut c_int,
        learning_rate: *mut c_float,
    ) -> c_int;

    // Thermal monitoring functions
    /// Read CPU temperature from sysfs (millidegrees Celsius)
    pub fn wvec_thermal_read(
        path: *const std::ffi::c_char,
        path_len: c_int,
        temp_mc: *mut c_int,
    ) -> c_int;

    /// Check if CPU is overheating (returns 1 if hot, 0 if OK, negative on error)
    pub fn wvec_thermal_check(
        path: *const std::ffi::c_char,
        path_len: c_int,
        threshold_c: c_int,
    ) -> c_int;

    /// Get CPU temperature in Celsius
    pub fn wvec_thermal_get_celsius(
        path: *const std::ffi::c_char,
        path_len: c_int,
        temp_c: *mut c_int,
    ) -> c_int;

    // Shutdown functions
    /// Request graceful shutdown
    pub fn wvec_shutdown_request();

    /// Check if shutdown was requested (returns 1 if yes, 0 if no)
    pub fn wvec_shutdown_check() -> c_int;

    /// Reset shutdown flag
    pub fn wvec_shutdown_reset();
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

    #[test]
    fn test_safe_array_scale() {
        let input = vec![1.0, 2.0, 3.0, 4.0];
        let result = array_scale(&input, 2.0).unwrap();

        assert_eq!(result.len(), 4);
        assert!((result[0] - 2.0).abs() < 1e-6);
        assert!((result[3] - 8.0).abs() < 1e-6);
    }

    #[test]
    fn test_safe_array_scale_empty() {
        let input: Vec<f32> = vec![];
        let result = array_scale(&input, 2.0);

        assert!(matches!(result, Err(FfiError::InvalidSize)));
    }

    #[test]
    fn test_safe_array_sum() {
        let arr = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = array_sum(&arr);
        assert!((result - 15.0).abs() < 1e-6);
    }

    #[test]
    fn test_safe_array_sum_empty() {
        let arr: Vec<f32> = vec![];
        let result = array_sum(&arr);
        assert!((result - 0.0).abs() < 1e-6);
    }

    #[test]
    fn test_model_init_free() {
        unsafe {
            // Initialize model
            let status = wvec_model_init(100, 64);
            assert_eq!(status, status::SUCCESS);
            assert_eq!(wvec_model_is_init(), 1);

            // Check dimensions
            let mut vocab_size: c_int = 0;
            let mut dim: c_int = 0;
            wvec_model_get_dims(&mut vocab_size, &mut dim);
            assert_eq!(vocab_size, 100);
            assert_eq!(dim, 64);

            // Free model
            wvec_model_free();
            assert_eq!(wvec_model_is_init(), 0);
        }
    }

    #[test]
    fn test_get_embedding() {
        unsafe {
            let status = wvec_model_init(10, 8);
            assert_eq!(status, status::SUCCESS);

            let mut embedding = [0.0f32; 8];
            let status = wvec_get_embedding(0, embedding.as_mut_ptr(), 8);
            assert_eq!(status, status::SUCCESS);

            // Embedding should be non-zero (randomly initialized)
            let sum: f32 = embedding.iter().map(|x| x.abs()).sum();
            assert!(sum > 0.0);

            wvec_model_free();
        }
    }

    #[test]
    fn test_train_pair() {
        unsafe {
            let status = wvec_model_init(100, 32);
            assert_eq!(status, status::SUCCESS);

            // Get embedding before training
            let mut emb_before = [0.0f32; 32];
            wvec_get_embedding(5, emb_before.as_mut_ptr(), 32);

            // Train one pair: center=5, context=10, negatives=[20, 30, 40]
            let neg_ids: [c_int; 3] = [20, 30, 40];
            let status = wvec_train_pair(5, 10, neg_ids.as_ptr(), 3, 0.025);
            assert_eq!(status, status::SUCCESS);

            // Get embedding after training
            let mut emb_after = [0.0f32; 32];
            wvec_get_embedding(5, emb_after.as_mut_ptr(), 32);

            // Embedding should have changed
            let diff: f32 = emb_before
                .iter()
                .zip(emb_after.iter())
                .map(|(a, b)| (a - b).abs())
                .sum();
            assert!(diff > 0.0, "Embedding should change after training");

            wvec_model_free();
        }
    }

    #[test]
    fn test_train_corpus() {
        unsafe {
            let status = wvec_model_init(1000, 64);
            assert_eq!(status, status::SUCCESS);

            // Simple corpus: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9] repeated
            let corpus: Vec<c_int> = (0..100).map(|i| i % 10).collect();

            // Negative sampling table (uniform for test)
            let neg_table: Vec<c_int> = (0..1000).map(|i| i % 1000).collect();

            let status = wvec_train_corpus(
                corpus.as_ptr(),
                corpus.len() as c_int,
                2, // window
                5, // n_neg
                neg_table.as_ptr(),
                neg_table.len() as c_int,
                0.025, // lr
            );
            assert_eq!(status, status::SUCCESS);

            wvec_model_free();
        }
    }

    #[test]
    fn test_checkpoint_save_load() {
        use std::fs;

        unsafe {
            // Initialize model
            let status = wvec_model_init(50, 16);
            assert_eq!(status, status::SUCCESS);

            // Get an embedding before save
            let mut emb_before = [0.0f32; 16];
            wvec_get_embedding(5, emb_before.as_mut_ptr(), 16);

            // Save checkpoint
            let path = "/tmp/wvec_test_checkpoint.bin";
            let status = wvec_checkpoint_save(
                path.as_ptr() as *const std::ffi::c_char,
                path.len() as c_int,
                42,   // epoch
                0.01, // learning_rate
            );
            assert_eq!(status, status::SUCCESS);

            // Free and verify model is gone
            wvec_model_free();
            assert_eq!(wvec_model_is_init(), 0);

            // Load checkpoint
            let mut epoch: c_int = 0;
            let mut lr: c_float = 0.0;
            let status = wvec_checkpoint_load(
                path.as_ptr() as *const std::ffi::c_char,
                path.len() as c_int,
                &mut epoch,
                &mut lr,
            );
            assert_eq!(status, status::SUCCESS);
            assert_eq!(wvec_model_is_init(), 1);
            assert_eq!(epoch, 42);
            assert!((lr - 0.01).abs() < 1e-6);

            // Check dimensions restored
            let mut vocab_size: c_int = 0;
            let mut dim: c_int = 0;
            wvec_model_get_dims(&mut vocab_size, &mut dim);
            assert_eq!(vocab_size, 50);
            assert_eq!(dim, 16);

            // Check embedding is identical
            let mut emb_after = [0.0f32; 16];
            wvec_get_embedding(5, emb_after.as_mut_ptr(), 16);
            for i in 0..16 {
                assert!(
                    (emb_before[i] - emb_after[i]).abs() < 1e-6,
                    "Embedding mismatch at index {}: {} vs {}",
                    i,
                    emb_before[i],
                    emb_after[i]
                );
            }

            wvec_model_free();
            fs::remove_file(path).ok();
        }
    }

    #[test]
    fn test_thermal_read() {
        let path = "/sys/class/thermal/thermal_zone10/temp";
        let mut temp_mc: c_int = 0;

        let status = unsafe {
            wvec_thermal_read(
                path.as_ptr() as *const std::ffi::c_char,
                path.len() as c_int,
                &mut temp_mc,
            )
        };

        assert_eq!(status, status::SUCCESS);
        // Temperature should be reasonable (0-120°C = 0-120000 millidegrees)
        assert!(
            temp_mc > 0 && temp_mc < 120000,
            "Unexpected temp: {}",
            temp_mc
        );
        println!("CPU temp: {}°C", temp_mc / 1000);
    }

    #[test]
    fn test_thermal_check_not_overheating() {
        let path = "/sys/class/thermal/thermal_zone10/temp";

        // Use high threshold (100°C) - should NOT be overheating
        let result = unsafe {
            wvec_thermal_check(
                path.as_ptr() as *const std::ffi::c_char,
                path.len() as c_int,
                100, // 100°C threshold
            )
        };

        assert_eq!(result, 0, "Should not be overheating at 100°C threshold");
    }

    #[test]
    fn test_thermal_get_celsius() {
        let path = "/sys/class/thermal/thermal_zone10/temp";
        let mut temp_c: c_int = 0;

        let status = unsafe {
            wvec_thermal_get_celsius(
                path.as_ptr() as *const std::ffi::c_char,
                path.len() as c_int,
                &mut temp_c,
            )
        };

        assert_eq!(status, status::SUCCESS);
        assert!(temp_c > 0 && temp_c < 120, "Unexpected temp: {}°C", temp_c);
    }

    #[test]
    fn test_thermal_read_invalid_path() {
        let path = "/nonexistent/thermal/path";
        let mut temp_mc: c_int = 0;

        let status = unsafe {
            wvec_thermal_read(
                path.as_ptr() as *const std::ffi::c_char,
                path.len() as c_int,
                &mut temp_mc,
            )
        };

        assert_eq!(status, status::ERR_FILE_IO);
    }
}
