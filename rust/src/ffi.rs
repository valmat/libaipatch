//! C ABI boundary for libaipatch.
//!
//! Exports the following C functions:
//! - aipatch_check
//! - aipatch_apply
//! - aipatch_result_free
//! - aipatch_version
//! - aipatch_abi_version

use std::ffi::{CStr, CString, c_char, c_int};
use std::path::Path;

use crate::errors::{AIPATCH_INTERNAL_ERROR, AIPATCH_INVALID_ARGUMENT, AIPATCH_OK};

/// The ABI major version. Increment on breaking changes.
pub const ABI_VERSION: c_int = 1;

/// The human-readable library version string.
pub const LIB_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");

/// Result struct returned by aipatch_check and aipatch_apply.
///
/// `code` contains an AIPATCH_* status code.
/// `message` is a null-terminated UTF-8 string (may be NULL).
/// `message_len` is the length of `message` in bytes, *without* the trailing \0.
///
/// Memory: if `message` is non-NULL, it was allocated by the library and
/// must be freed via `aipatch_result_free`.
#[repr(C)]
pub struct AipatchResult {
    pub code: c_int,
    pub message: *mut c_char,
    pub message_len: usize,
}

impl AipatchResult {
    /// Create a success result with an optional message.
    fn ok(message: Option<String>) -> Self {
        Self::with_code(AIPATCH_OK, message)
    }

    /// Create an error result.
    fn err(code: i32, message: String) -> Self {
        Self::with_code(code, Some(message))
    }

    fn with_code(code: i32, message: Option<String>) -> Self {
        match message {
            None => AipatchResult {
                code,
                message: std::ptr::null_mut(),
                message_len: 0,
            },
            Some(msg) => {
                let len = msg.len();
                match CString::new(msg) {
                    Ok(cstr) => {
                        let ptr = cstr.into_raw();
                        AipatchResult {
                            code,
                            message: ptr,
                            message_len: len,
                        }
                    }
                    Err(_) => {
                        // Non-UTF-8: fallback to a safe message.
                        let fallback = CString::new("(message encoding error)").unwrap();
                        let fallback_len = fallback.as_bytes().len();
                        AipatchResult {
                            code,
                            message: fallback.into_raw(),
                            message_len: fallback_len,
                        }
                    }
                }
            }
        }
    }

    /// Write the result into a caller-provided `*mut AipatchResult`.
    ///
    /// # Safety
    /// `out` must be a valid non-null pointer.
    unsafe fn write_to(self, out: *mut AipatchResult) {
        unsafe {
            (*out).code = self.code;
            (*out).message = self.message;
            (*out).message_len = self.message_len;
        }
    }
}

/// Parse a raw C string slice into a &str, returning AIPATCH_INVALID_ARGUMENT on error.
///
/// # Safety
/// `ptr` must be valid for `len` bytes if non-null.
unsafe fn c_str_to_rust<'a>(
    ptr: *const c_char,
    len: usize,
    field_name: &str,
) -> Result<&'a str, AipatchResult> {
    if ptr.is_null() {
        return Err(AipatchResult::err(
            AIPATCH_INVALID_ARGUMENT,
            format!("{field_name} pointer is null"),
        ));
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
    std::str::from_utf8(bytes).map_err(|_| {
        AipatchResult::err(
            AIPATCH_INVALID_ARGUMENT,
            format!("{field_name} is not valid UTF-8"),
        )
    })
}

/// Validate patch and check applicability without writing to disk.
///
/// Returns 0 on ABI-level success (caller should then check `out->code`).
/// Returns non-zero if `out` is null or a catastrophic ABI-level failure occurred.
///
/// # Safety
/// - `patch` must point to `patch_len` valid UTF-8 bytes.
/// - `root_dir` must point to `root_dir_len` valid UTF-8 bytes.
/// - `out` must be a valid non-null pointer to an `AipatchResult`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipatch_check(
    patch: *const c_char,
    patch_len: usize,
    root_dir: *const c_char,
    root_dir_len: usize,
    out: *mut AipatchResult,
) -> c_int {
    if out.is_null() {
        return -1;
    }

    let result = unsafe { run_check(patch, patch_len, root_dir, root_dir_len) };
    unsafe { result.write_to(out) };
    0
}

unsafe fn run_check(
    patch: *const c_char,
    patch_len: usize,
    root_dir: *const c_char,
    root_dir_len: usize,
) -> AipatchResult {
    let patch_str = match unsafe { c_str_to_rust(patch, patch_len, "patch") } {
        Ok(s) => s,
        Err(r) => return r,
    };
    let root_str = match unsafe { c_str_to_rust(root_dir, root_dir_len, "root_dir") } {
        Ok(s) => s,
        Err(r) => return r,
    };
    let root_path = Path::new(root_str);

    match crate::engine::check(patch_str, root_path) {
        Ok(()) => AipatchResult::ok(None),
        Err(e) => AipatchResult::err(e.abi_code(), e.to_string()),
    }
}

/// Apply patch to the filesystem rooted at `root_dir`.
///
/// Validates first; writes only after full successful validation.
/// Returns 0 on ABI-level success (caller should then check `out->code`).
///
/// # Safety
/// Same as `aipatch_check`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipatch_apply(
    patch: *const c_char,
    patch_len: usize,
    root_dir: *const c_char,
    root_dir_len: usize,
    out: *mut AipatchResult,
) -> c_int {
    if out.is_null() {
        return -1;
    }

    let result = unsafe { run_apply(patch, patch_len, root_dir, root_dir_len) };
    unsafe { result.write_to(out) };
    0
}

unsafe fn run_apply(
    patch: *const c_char,
    patch_len: usize,
    root_dir: *const c_char,
    root_dir_len: usize,
) -> AipatchResult {
    let patch_str = match unsafe { c_str_to_rust(patch, patch_len, "patch") } {
        Ok(s) => s,
        Err(r) => return r,
    };
    let root_str = match unsafe { c_str_to_rust(root_dir, root_dir_len, "root_dir") } {
        Ok(s) => s,
        Err(r) => return r,
    };
    let root_path = Path::new(root_str);

    match crate::engine::apply(patch_str, root_path) {
        Ok(res) => AipatchResult::ok(Some(res.summary)),
        Err(e) => AipatchResult::err(e.abi_code(), e.to_string()),
    }
}

/// Free memory allocated in an AipatchResult.
///
/// Safe to call on a zeroed/already-freed result (idempotent).
///
/// # Safety
/// `result` must be either null or a valid pointer to an `AipatchResult`
/// whose `message` field was allocated by `aipatch_check` or `aipatch_apply`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipatch_result_free(result: *mut AipatchResult) {
    if result.is_null() {
        return;
    }
    unsafe {
        let r = &mut *result;
        if !r.message.is_null() {
            // Reconstruct the CString to free it properly.
            drop(CString::from_raw(r.message));
            r.message = std::ptr::null_mut();
            r.message_len = 0;
        }
    }
}

/// Return a static null-terminated string with the library version.
///
/// The returned pointer is valid for the lifetime of the process.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipatch_version() -> *const c_char {
    LIB_VERSION.as_ptr() as *const c_char
}

/// Return the ABI major version number.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn aipatch_abi_version() -> c_int {
    ABI_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;
    use tempfile::tempdir;

    fn call_check(patch: &str, root: &str) -> (c_int, AipatchResult) {
        let patch_c = CString::new(patch).unwrap();
        let root_c = CString::new(root).unwrap();
        let mut out = AipatchResult {
            code: -1,
            message: std::ptr::null_mut(),
            message_len: 0,
        };
        let rc = unsafe {
            aipatch_check(
                patch_c.as_ptr(),
                patch.len(),
                root_c.as_ptr(),
                root.len(),
                &mut out,
            )
        };
        (rc, out)
    }

    fn call_apply(patch: &str, root: &str) -> (c_int, AipatchResult) {
        let patch_c = CString::new(patch).unwrap();
        let root_c = CString::new(root).unwrap();
        let mut out = AipatchResult {
            code: -1,
            message: std::ptr::null_mut(),
            message_len: 0,
        };
        let rc = unsafe {
            aipatch_apply(
                patch_c.as_ptr(),
                patch.len(),
                root_c.as_ptr(),
                root.len(),
                &mut out,
            )
        };
        (rc, out)
    }

    #[test]
    fn test_check_success() {
        let dir = tempdir().unwrap();
        let patch = "*** Begin Patch\n*** Add File: new.txt\n+hi\n*** End Patch";
        let (rc, out) = call_check(patch, dir.path().to_str().unwrap());
        assert_eq!(rc, 0);
        assert_eq!(out.code, AIPATCH_OK);
        unsafe { aipatch_result_free(&mut out as *mut _ as *mut AipatchResult) };
    }

    #[test]
    fn test_check_parse_error() {
        let dir = tempdir().unwrap();
        let (rc, mut out) = call_check("bad patch", dir.path().to_str().unwrap());
        assert_eq!(rc, 0);
        assert_ne!(out.code, AIPATCH_OK);
        unsafe { aipatch_result_free(&mut out) };
    }

    #[test]
    fn test_apply_success() {
        let dir = tempdir().unwrap();
        let patch = "*** Begin Patch\n*** Add File: hello.txt\n+world\n*** End Patch";
        let root = dir.path().to_str().unwrap();
        let (rc, mut out) = call_apply(patch, root);
        assert_eq!(rc, 0);
        assert_eq!(out.code, AIPATCH_OK);
        unsafe { aipatch_result_free(&mut out) };
        assert!(dir.path().join("hello.txt").exists());
    }

    #[test]
    fn test_apply_path_traversal() {
        let dir = tempdir().unwrap();
        let patch = "*** Begin Patch\n*** Add File: ../../evil\n+x\n*** End Patch";
        let (rc, mut out) = call_apply(patch, dir.path().to_str().unwrap());
        assert_eq!(rc, 0);
        assert_eq!(out.code, crate::errors::AIPATCH_PATH_VIOLATION);
        unsafe { aipatch_result_free(&mut out) };
    }

    #[test]
    fn test_result_free_idempotent() {
        let dir = tempdir().unwrap();
        let patch = "*** Begin Patch\n*** Add File: x.txt\n+y\n*** End Patch";
        let (_, mut out) = call_check(patch, dir.path().to_str().unwrap());
        // Double-free should be safe (message becomes null after first free).
        unsafe { aipatch_result_free(&mut out) };
        unsafe { aipatch_result_free(&mut out) };
    }

    #[test]
    fn test_version_functions() {
        let ver = unsafe { CStr::from_ptr(aipatch_version()) };
        assert!(!ver.to_str().unwrap().is_empty());
        assert_eq!(unsafe { aipatch_abi_version() }, ABI_VERSION);
    }

    #[test]
    fn test_null_out_returns_minus_one() {
        let patch = CString::new("*** Begin Patch\n*** End Patch").unwrap();
        let root = CString::new("/tmp").unwrap();
        let rc = unsafe {
            aipatch_check(patch.as_ptr(), 30, root.as_ptr(), 4, std::ptr::null_mut())
        };
        assert_eq!(rc, -1);
    }
}
