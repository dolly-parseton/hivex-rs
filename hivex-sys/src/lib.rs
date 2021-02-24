#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(dead_code)]

// Add in bindings.rs
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Mod
mod error;

// Uses
use byteorder::{ByteOrder, LittleEndian};
use std::{collections::HashMap, ffi, path, ptr};
// Structs
pub type Result<T> = std::result::Result<T, error::Error>;
pub struct Hive {
    handle: *mut hive_h,
}

// Functions

/// Wraps 'hivex_open' function, returns a hive_h
pub fn open<P>(path: P) -> Result<*mut hive_h>
where
    P: AsRef<path::Path>,
{
    // Does the path exist
    match path.as_ref().exists() {
        // No return appropriate error
        false => return Err(error::Error::hive_does_not_exist(path)),
        true => {
            // Get the hive handle
            let handle = match path.as_ref().to_str() {
                Some(s) => unsafe {
                    // Run the hivex_open function in unsafe
                    hivex_open(
                        ffi::CString::new(s)
                            .map_err(|e| error::Error::ffi_error(e))?
                            .as_ptr(),
                        1,
                    )
                },
                None => return Err(error::Error::string_convert_error()),
            };
            return match handle.is_null() {
                false => Ok(handle),
                true => Err(error::Error::hivex_error("hivex_open")),
            };
        }
    }
}

/// Wraps 'hivex_close' function, returns Ok if successful and an error on failure
/// * Todo, add error parsing from errno.
pub fn close(hive: *mut hive_h) -> Result<()> {
    match unsafe { hivex_close(hive) } == 0 {
        true => Ok(()),
        false => Err(error::Error::hivex_error("hivex_close")),
    }
}

/// Wraps 'hivex_root' function, returns Ok if successful and an error on failure
/// * Todo, add error parsing from errno.
pub fn root(hive: *mut hive_h) -> Result<hive_node_h> {
    match unsafe { hivex_root(hive) } {
        0 => Err(error::Error::hivex_error("hivex_root")),
        n => Ok(n),
    }
}

// Does not work on test_data2 for unknown reasons.
// /// Wraps 'hivex_last_modified' function, returns Ok if successful and an error on failure
// /// * Todo, add error parsing from errno.
// pub fn last_modified(hive: *mut hive_h) -> Result<i64> {
//     match unsafe { hivex_last_modified(hive) } {
//         0 => Err(error::Error::hivex_error("hivex_last_modified")),
//         n => Ok(n),
//     }
// }

/// Wraps 'hivex_node_name' function, returns Ok if successful and an error on failure
/// * Todo, add error parsing from errno.
pub fn node_name(hive: *mut hive_h, node: u64) -> Result<String> {
    let res = unsafe { hivex_node_name(hive, node) };
    match res.is_null() {
        true => Err(error::Error::hivex_error("hivex_node_name")),
        false => match unsafe { ffi::CString::from_raw(res) }.into_string() {
            Ok(s) => Ok(s),
            Err(e) => Err(error::Error::ffi_error(e)),
        },
    }
}

/// Wraps 'hivex_node_timestamp' function, returns Ok if successful.
/// No error handling here however that may need to be readdressed after testing.
pub fn node_last_modified(hive: *mut hive_h, node: u64) -> Result<i64> {
    Ok(unsafe { hivex_node_timestamp(hive, node) })
}

/// Wraps 'hivex_node_children' function, returns Ok if successful.
/// Also contains 'hivex_node_children'
/// No error handling here however that may need to be readdressed after testing.
pub fn node_children(hive: *mut hive_h, node: u64) -> Result<Vec<*mut hive_node_h>> {
    let res: *mut u64 = unsafe { hivex_node_children(hive, node) };
    let n = unsafe { hivex_node_children(hive, node) as usize };
    let slice = unsafe { std::slice::from_raw_parts(res, n) };
    Ok(slice.to_owned())
}

/// Wraps 'hivex_node_parent' function, returns Ok if successful.
/// * Todo, add error parsing from errno.
pub fn node_parent(hive: *mut hive_h, node: u64) -> Result<u64> {
    match unsafe { hivex_node_parent(hive, node) } {
        0 => Err(error::Error::hivex_error("hivex_node_parent")),
        n => Ok(n),
    }
}

// /// Wraps 'hivex_node_values' function, returns Ok if successful.
// /// Also contains 'hivex_node_nr_values'
// /// No error handling here however that may need to be readdressed after testing.
// pub fn node_values(hive: *mut hive_h, node: u64) -> Result<Vec<u64>> {
//     let res: *mut u64 = unsafe { hivex_node_values(hive, node) };
//     let n = unsafe { hivex_node_nr_values(hive, node) as usize };
//     let slice = unsafe { std::slice::from_raw_parts(res, n) };
//     Ok(slice.to_owned())
// }

/// Wraps 'hivex_value_key' function, returns Ok if successful and an error on failure
/// * Todo, add error parsing from errno.
pub fn value_key(hive: *mut hive_h, value: u64) -> Result<String> {
    let res = unsafe { hivex_value_key(hive, value) };
    match res.is_null() {
        true => Err(error::Error::hivex_error("hivex_value_key")),
        false => match unsafe { ffi::CString::from_raw(res) }.into_string() {
            Ok(s) => Ok(s),
            Err(e) => Err(error::Error::ffi_error(e)),
        },
    }
}

// Do I need todo type as it's own thing?

pub fn node_values(hive: *mut hive_h, node: u64) {
    use std::convert::TryInto;
    let n: usize = unsafe { hivex_node_nr_values(hive, node).try_into().unwrap() };
    let res = unsafe { hivex_node_values(hive, node) };
    println!("{:?} {:?}", res, n);
    let slice = unsafe { std::slice::from_raw_parts(res, n) };
    println!("{:?}", slice);
}

/// Wraps 'hivex_value_value' function, returns Ok if successful and an error on failure
/// * Todo, add error parsing from errno.
pub fn value_value(hive: *mut hive_h, value: u64) -> Result<String> {
    // Get type (u32)
    let type_ptr = std::ptr::null_mut();
    let len_ptr = std::ptr::null_mut();
    // let value_type = unsafe { hivex_value_type(hive, value, type_ptr, len_ptr) };
    // Get size

    // Get value value as Vec<u8>, can cast to type in rs crate
    let res = unsafe { hivex_value_value(hive, value, type_ptr, len_ptr) };
    match res.is_null() {
        true => Err(error::Error::hivex_error("hivex_value_value")),
        false => match unsafe { ffi::CString::from_raw(res) }.into_string() {
            Ok(s) => Ok(s),
            Err(e) => Err(error::Error::ffi_error(e)),
        },
    }
}

pub fn convert_windows_to_epoch(ticks: i64) -> Result<u64> {
    use std::convert::TryInto;
    Ok((ticks / 10000000 - 11644473600).try_into()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hive_open() {
        let res = open("../../test_data2");
        println!("{:?}", res);
        assert!(res.is_ok());
    }

    #[test]
    fn hive_close() {
        let hive: Result<*mut hive_h> = open("../../test_data2");
        assert!(hive.is_ok());
        assert!(close(hive.unwrap()).is_ok());
    }

    #[test]
    fn hive_root() {
        let hive: Result<*mut hive_h> = open("../../test_data2");
        assert!(hive.is_ok());
        let hive = hive.unwrap();
        let root = root(hive);
        assert!(root.is_ok());
        assert!(close(hive).is_ok());
    }

    // Disabled until bug is understood
    // #[test]
    // fn hivex_last_modified() {
    //     let hive: Result<*mut hive_h> = open("../../test_data2");
    //     assert!(hive.is_ok());
    //     let hive = hive.unwrap();
    //     let last_modified = last_modified(hive);
    //     println!("{:?}", last_modified);
    //     assert!(last_modified.is_ok());
    //     assert!(close(hive).is_err());
    // }

    #[test]
    fn hive_node_last_modified() {
        let hive: Result<*mut hive_h> = open("../../test_data2");
        assert!(hive.is_ok());
        let hive = hive.unwrap();
        let root = root(hive);
        assert!(root.is_ok());
        let root = root.unwrap();
        let lm = node_last_modified(hive, root);
        println!("{:?}", lm);
        assert!(lm.is_ok());
        println!("{:?}", convert_windows_to_epoch(lm.unwrap()));
        assert!(close(hive).is_ok());
    }

    #[test]
    fn hive_values() {
        let hive: Result<*mut hive_h> = open("../../test_data2");
        assert!(hive.is_ok());
        let hive = hive.unwrap();
        let root = root(hive);
        assert!(root.is_ok());
        let root = root.unwrap();

        let child = node_children(hive, root);
        assert!(child.is_ok());
        let child = child.unwrap()[0];

        let values = node_values(hive, child);
        assert!(close(hive).is_err());
    }
}
