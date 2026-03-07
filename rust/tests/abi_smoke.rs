mod support;

use std::ffi::{c_char, CStr, CString};

use aipatch::errors::{AIPATCH_INVALID_ARGUMENT, AIPATCH_OK};
use aipatch::ffi::{
    aipatch_abi_version, aipatch_apply, aipatch_check, aipatch_result_free, aipatch_version,
    AipatchResult, ABI_VERSION,
};
use support::TempDir;

#[test]
fn abi_check_and_apply_smoke() {
    let dir = TempDir::new().unwrap();
    let root = CString::new(dir.path().to_str().unwrap()).unwrap();
    let patch = CString::new("*** Begin Patch\n*** Add File: smoke.txt\n+ok\n*** End Patch").unwrap();

    let mut check_out = empty_result();
    let check_rc = unsafe {
        aipatch_check(
            patch.as_ptr(),
            patch.as_bytes().len(),
            root.as_ptr(),
            root.as_bytes().len(),
            &mut check_out,
        )
    };
    assert_eq!(check_rc, 0);
    assert_eq!(check_out.code, AIPATCH_OK);
    unsafe { aipatch_result_free(&mut check_out) };

    let mut apply_out = empty_result();
    let apply_rc = unsafe {
        aipatch_apply(
            patch.as_ptr(),
            patch.as_bytes().len(),
            root.as_ptr(),
            root.as_bytes().len(),
            &mut apply_out,
        )
    };
    assert_eq!(apply_rc, 0);
    assert_eq!(apply_out.code, AIPATCH_OK);
    assert!(!apply_out.message.is_null());
    let summary = unsafe { CStr::from_ptr(apply_out.message) }.to_str().unwrap();
    assert!(summary.contains("status: ok"));
    assert!(summary.contains("operations:"));
    assert!(summary.contains("A "));
    unsafe { aipatch_result_free(&mut apply_out) };

    assert_eq!(std::fs::read_to_string(dir.path().join("smoke.txt")).unwrap(), "ok\n");
}

#[test]
fn abi_invalid_utf8_contract() {
    let dir = TempDir::new().unwrap();
    let root = dir.path().to_str().unwrap().as_bytes();
    let invalid_patch = [0xff, 0xfe];
    let mut out = empty_result();

    let rc = unsafe {
        aipatch_check(
            invalid_patch.as_ptr().cast::<c_char>(),
            invalid_patch.len(),
            root.as_ptr().cast::<c_char>(),
            root.len(),
            &mut out,
        )
    };
    assert_eq!(rc, 0);
    assert_eq!(out.code, AIPATCH_INVALID_ARGUMENT);
    let message = unsafe { CStr::from_ptr(out.message) }.to_str().unwrap();
    assert!(message.contains("patch is not valid UTF-8"));
    unsafe { aipatch_result_free(&mut out) };
}

#[test]
fn abi_version_functions_are_stable() {
    let version = unsafe { CStr::from_ptr(aipatch_version()) };
    assert!(!version.to_str().unwrap().is_empty());
    assert_eq!(aipatch_abi_version(), ABI_VERSION);
}

fn empty_result() -> AipatchResult {
    AipatchResult {
        code: -1,
        message: std::ptr::null_mut(),
        message_len: 0,
    }
}
