//! C ABI boundary for libaipatch.
//!
//! Exports the following C functions:
//! - aipatch_check
//! - aipatch_apply
//! - aipatch_result_free
//! - aipatch_version
//! - aipatch_abi_version

use std::collections::HashSet;
use std::ffi::{c_char, c_int, CString};
#[cfg(test)]
use std::ffi::CStr;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use crate::errors::{AIPATCH_INVALID_ARGUMENT, AIPATCH_OK};
#[cfg(test)]
use crate::errors::{AIPATCH_PATH_VIOLATION, AIPATCH_UNSUPPORTED};

pub const ABI_VERSION: c_int = 1;
pub const LIB_VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");

#[repr(C)]
pub struct AipatchResult {
    pub code: c_int,
    pub message: *mut c_char,
    pub message_len: usize,
}

impl AipatchResult {
    fn ok(message: Option<String>) -> Self {
        Self::with_code(AIPATCH_OK, message)
    }

    fn err(code: i32, message: String) -> Self {
        Self::with_code(code, Some(message))
    }

    fn with_code(code: i32, message: Option<String>) -> Self {
        match message {
            None => Self {
                code,
                message: std::ptr::null_mut(),
                message_len: 0,
            },
            Some(message) => {
                let message = sanitize_message(message);
                let message_len = message.len();
                let cstring = CString::new(message).expect("sanitized message must not contain NUL");
                let ptr = cstring.into_raw();
                register_owned_message(ptr);
                Self {
                    code,
                    message: ptr,
                    message_len,
                }
            }
        }
    }

    unsafe fn write_to(self, out: *mut AipatchResult) {
        free_owned_message_if_tracked((*out).message);
        (*out).code = self.code;
        (*out).message = self.message;
        (*out).message_len = self.message_len;
    }
}

fn owned_messages() -> &'static Mutex<HashSet<usize>> {
    static OWNED_MESSAGES: OnceLock<Mutex<HashSet<usize>>> = OnceLock::new();
    OWNED_MESSAGES.get_or_init(|| Mutex::new(HashSet::new()))
}

fn register_owned_message(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    let mut registry = match owned_messages().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    registry.insert(ptr as usize);
}

fn take_owned_message(ptr: *mut c_char) -> bool {
    if ptr.is_null() {
        return false;
    }
    let mut registry = match owned_messages().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    registry.remove(&(ptr as usize))
}

unsafe fn free_owned_message_if_tracked(ptr: *mut c_char) {
    if take_owned_message(ptr) {
        drop(CString::from_raw(ptr));
    }
}

fn sanitize_message(message: String) -> String {
    if message.as_bytes().contains(&0) {
        message.replace('\0', "�")
    } else {
        message
    }
}

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

    let bytes = std::slice::from_raw_parts(ptr.cast::<u8>(), len);
    std::str::from_utf8(bytes).map_err(|_| {
        AipatchResult::err(
            AIPATCH_INVALID_ARGUMENT,
            format!("{field_name} is not valid UTF-8"),
        )
    })
}

#[no_mangle]
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

    let result = run_check(patch, patch_len, root_dir, root_dir_len);
    result.write_to(out);
    0
}

unsafe fn run_check(
    patch: *const c_char,
    patch_len: usize,
    root_dir: *const c_char,
    root_dir_len: usize,
) -> AipatchResult {
    let patch_str = match c_str_to_rust(patch, patch_len, "patch") {
        Ok(value) => value,
        Err(result) => return result,
    };
    let root_dir_str = match c_str_to_rust(root_dir, root_dir_len, "root_dir") {
        Ok(value) => value,
        Err(result) => return result,
    };

    match crate::engine::check(patch_str, Path::new(root_dir_str)) {
        Ok(()) => AipatchResult::ok(None),
        Err(err) => AipatchResult::err(err.abi_code(), err.to_string()),
    }
}

#[no_mangle]
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

    let result = run_apply(patch, patch_len, root_dir, root_dir_len);
    result.write_to(out);
    0
}

unsafe fn run_apply(
    patch: *const c_char,
    patch_len: usize,
    root_dir: *const c_char,
    root_dir_len: usize,
) -> AipatchResult {
    let patch_str = match c_str_to_rust(patch, patch_len, "patch") {
        Ok(value) => value,
        Err(result) => return result,
    };
    let root_dir_str = match c_str_to_rust(root_dir, root_dir_len, "root_dir") {
        Ok(value) => value,
        Err(result) => return result,
    };

    match crate::engine::apply(patch_str, Path::new(root_dir_str)) {
        Ok(result) => AipatchResult::ok(Some(result.summary)),
        Err(err) => AipatchResult::err(err.abi_code(), err.to_string()),
    }
}

#[no_mangle]
pub unsafe extern "C" fn aipatch_result_free(result: *mut AipatchResult) {
    if result.is_null() {
        return;
    }

    let result = &mut *result;
    free_owned_message_if_tracked(result.message);
    result.message = std::ptr::null_mut();
    result.message_len = 0;
}

#[no_mangle]
pub extern "C" fn aipatch_version() -> *const c_char {
    LIB_VERSION.as_ptr().cast::<c_char>()
}

#[no_mangle]
pub extern "C" fn aipatch_abi_version() -> c_int {
    ABI_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    fn call_check(patch: &[u8], root: &[u8]) -> (c_int, AipatchResult) {
        let mut out = AipatchResult {
            code: -1,
            message: std::ptr::null_mut(),
            message_len: 999,
        };
        let rc = unsafe {
            aipatch_check(
                patch.as_ptr().cast::<c_char>(),
                patch.len(),
                root.as_ptr().cast::<c_char>(),
                root.len(),
                &mut out,
            )
        };
        (rc, out)
    }

    fn call_apply(patch: &str, root: &str) -> (c_int, AipatchResult) {
        let patch = CString::new(patch).unwrap();
        let root = CString::new(root).unwrap();
        let mut out = AipatchResult {
            code: -1,
            message: std::ptr::null_mut(),
            message_len: 999,
        };
        let rc = unsafe {
            aipatch_apply(
                patch.as_ptr(),
                patch.as_bytes().len(),
                root.as_ptr(),
                root.as_bytes().len(),
                &mut out,
            )
        };
        (rc, out)
    }

    #[test]
    fn test_check_success() {
        let dir = TempDir::new().unwrap();
        let patch = b"*** Begin Patch\n*** Add File: new.txt\n+hi\n*** End Patch";
        let root = dir.path().to_str().unwrap().as_bytes();
        let (rc, mut out) = call_check(patch, root);
        assert_eq!(rc, 0);
        assert_eq!(out.code, AIPATCH_OK);
        unsafe { aipatch_result_free(&mut out) };
    }

    #[test]
    fn test_check_parse_error() {
        let dir = TempDir::new().unwrap();
        let (rc, mut out) = call_check(b"bad patch", dir.path().to_str().unwrap().as_bytes());
        assert_eq!(rc, 0);
        assert_ne!(out.code, AIPATCH_OK);
        assert!(!out.message.is_null());
        unsafe { aipatch_result_free(&mut out) };
    }

    #[test]
    fn test_apply_success() {
        let dir = TempDir::new().unwrap();
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
        let dir = TempDir::new().unwrap();
        let patch = "*** Begin Patch\n*** Add File: ../../evil\n+x\n*** End Patch";
        let (rc, mut out) = call_apply(patch, dir.path().to_str().unwrap());
        assert_eq!(rc, 0);
        assert_eq!(out.code, AIPATCH_PATH_VIOLATION);
        unsafe { aipatch_result_free(&mut out) };
    }

    #[test]
    fn test_result_free_idempotent() {
        let dir = TempDir::new().unwrap();
        let patch = b"*** Begin Patch\n*** Add File: x.txt\n+y\n*** End Patch";
        let root = dir.path().to_str().unwrap().as_bytes();
        let (_, mut out) = call_check(patch, root);
        unsafe { aipatch_result_free(&mut out) };
        unsafe { aipatch_result_free(&mut out) };
    }

    #[test]
    fn test_version_functions() {
        let version = unsafe { CStr::from_ptr(aipatch_version()) };
        assert!(!version.to_str().unwrap().is_empty());
        assert_eq!(aipatch_abi_version(), ABI_VERSION);
    }

    #[test]
    fn test_null_out_returns_minus_one() {
        let patch = CString::new("*** Begin Patch\n*** End Patch").unwrap();
        let root = CString::new("/tmp").unwrap();
        let rc = unsafe {
            aipatch_check(
                patch.as_ptr(),
                patch.as_bytes().len(),
                root.as_ptr(),
                root.as_bytes().len(),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(rc, -1);
    }

    #[test]
    fn test_invalid_utf8_is_invalid_argument() {
        let dir = TempDir::new().unwrap();
        let patch = [0xff, 0xfe, 0xfd];
        let (rc, mut out) = call_check(&patch, dir.path().to_str().unwrap().as_bytes());
        assert_eq!(rc, 0);
        assert_eq!(out.code, AIPATCH_INVALID_ARGUMENT);
        let message = unsafe { CStr::from_ptr(out.message) }.to_str().unwrap().to_owned();
        assert!(message.contains("patch is not valid UTF-8"));
        unsafe { aipatch_result_free(&mut out) };
    }

    #[test]
    fn test_non_utf8_target_file_is_reported_as_unsupported() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("f.txt"), [0xff, 0xfe, 0xfd]).unwrap();

        let patch = CString::new("*** Begin Patch\n*** Update File: f.txt\n@@\n-old\n+new\n*** End Patch").unwrap();
        let root = CString::new(dir.path().to_str().unwrap()).unwrap();
        let mut out = AipatchResult {
            code: -1,
            message: std::ptr::null_mut(),
            message_len: 999,
        };

        let rc = unsafe {
            aipatch_check(
                patch.as_ptr(),
                patch.as_bytes().len(),
                root.as_ptr(),
                root.as_bytes().len(),
                &mut out,
            )
        };

        assert_eq!(rc, 0);
        assert_eq!(out.code, AIPATCH_UNSUPPORTED);
        let message = unsafe { CStr::from_ptr(out.message) }.to_str().unwrap().to_owned();
        assert!(message.contains("tag: unsupported.file.non_utf8"));
        assert!(message.contains("non-UTF-8"));
        unsafe { aipatch_result_free(&mut out) };
    }

    #[test]
    fn test_path_violation_message_is_agent_friendly() {
        let dir = TempDir::new().unwrap();
        let patch = CString::new("*** Begin Patch\n*** Add File: ../../evil\n+x\n*** End Patch").unwrap();
        let root = CString::new(dir.path().to_str().unwrap()).unwrap();
        let mut out = AipatchResult {
            code: -1,
            message: std::ptr::null_mut(),
            message_len: 999,
        };

        let rc = unsafe {
            aipatch_check(
                patch.as_ptr(),
                patch.as_bytes().len(),
                root.as_ptr(),
                root.as_bytes().len(),
                &mut out,
            )
        };

        assert_eq!(rc, 0);
        assert_eq!(out.code, AIPATCH_PATH_VIOLATION);
        let message = unsafe { CStr::from_ptr(out.message) }.to_str().unwrap().to_owned();
        assert!(message.contains("tag: path.traversal_detected"));
        assert!(message.contains("root_dir:"));
        unsafe { aipatch_result_free(&mut out) };
    }

    #[test]
    fn test_reusing_out_replaces_previous_owned_message() {
        let dir = TempDir::new().unwrap();
        let root = CString::new(dir.path().to_str().unwrap()).unwrap();
        let bad_patch = CString::new("bad patch").unwrap();
        let good_patch = CString::new("*** Begin Patch\n*** Add File: ok.txt\n+hi\n*** End Patch").unwrap();
        let mut out = AipatchResult {
            code: -1,
            message: std::ptr::null_mut(),
            message_len: 0,
        };

        let rc1 = unsafe {
            aipatch_check(
                bad_patch.as_ptr(),
                bad_patch.as_bytes().len(),
                root.as_ptr(),
                root.as_bytes().len(),
                &mut out,
            )
        };
        assert_eq!(rc1, 0);
        assert!(!out.message.is_null());
        assert!(out.message_len > 0);

        let rc2 = unsafe {
            aipatch_check(
                good_patch.as_ptr(),
                good_patch.as_bytes().len(),
                root.as_ptr(),
                root.as_bytes().len(),
                &mut out,
            )
        };
        assert_eq!(rc2, 0);
        assert_eq!(out.code, AIPATCH_OK);
        assert!(out.message.is_null());
        assert_eq!(out.message_len, 0);
        unsafe { aipatch_result_free(&mut out) };
    }

    #[test]
    fn test_reusing_out_with_foreign_message_is_safe() {
        let dir = TempDir::new().unwrap();
        let patch = b"*** Begin Patch\n*** Add File: new.txt\n+hi\n*** End Patch";
        let root = dir.path().to_str().unwrap().as_bytes();
        static FOREIGN_MESSAGE: &[u8] = b"foreign\0";
        let mut out = AipatchResult {
            code: 123,
            message: FOREIGN_MESSAGE.as_ptr().cast::<c_char>() as *mut c_char,
            message_len: 7,
        };

        let rc = unsafe {
            aipatch_check(
                patch.as_ptr().cast::<c_char>(),
                patch.len(),
                root.as_ptr().cast::<c_char>(),
                root.len(),
                &mut out,
            )
        };
        assert_eq!(rc, 0);
        assert_eq!(out.code, AIPATCH_OK);
        assert!(out.message.is_null());
        assert_eq!(out.message_len, 0);
        unsafe { aipatch_result_free(&mut out) };
    }
}
