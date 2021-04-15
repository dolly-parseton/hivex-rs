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
use std::{ffi, path};
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
/// Wraps 'hivex_last_modified' function, returns Ok if successful and an error on failure
/// * Todo, add error parsing from errno.
pub fn last_modified(hive: *mut hive_h) -> Result<i64> {
    // println!("{:?}", unsafe { hivex_last_modified(hive) });
    Ok(unsafe { hivex_last_modified(hive) })
    // match unsafe { hivex_last_modified(hive) } {
    //     0 => Err(error::Error::hivex_error("hivex_last_modified")),
    //     n => Ok(n),
    // }
}

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
pub fn node_children(hive: *mut hive_h, node: u64) -> Result<Vec<hive_node_h>> {
    let res = unsafe { hivex_node_children(hive, node) };
    let n = unsafe { hivex_node_nr_children(hive, node) };
    // Fail if null
    match res.is_null() && n != 0 {
        true => Err(error::Error::hivex_error("hivex_node_children")),
        false => {
            let slice = unsafe { std::slice::from_raw_parts(res, n as usize) };
            // println!("{:?}", slice);
            Ok(slice.to_vec())
        }
    }
}

/// Wraps 'hivex_node_parent' function, returns Ok if successful.
/// * Todo, add error parsing from errno.
pub fn node_parent(hive: *mut hive_h, node: hive_node_h) -> Result<u64> {
    match unsafe { hivex_node_parent(hive, node) } {
        0 => Err(error::Error::hivex_error("hivex_node_parent")),
        n => Ok(n),
    }
}

/// Wraps 'hivex_node_values' function, returns Ok if successful.
/// Also contains 'hivex_node_nr_values'
pub fn node_values(hive: *mut hive_h, node: hive_node_h) -> Result<Vec<hive_value_h>> {
    let res = unsafe { hivex_node_values(hive, node) };
    let n = unsafe { hivex_node_nr_values(hive, node) };
    match res.is_null() {
        true => Err(error::Error::hivex_error("hivex_node_values")),
        false => {
            let slice = unsafe { std::slice::from_raw_parts(res, n as usize) };
            Ok(slice.to_vec())
        }
    }
}

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

/// Wraps 'hivex_value_type' function, returns Ok with the type and len if successful and an error on failure
/// * Todo, add error parsing from errno.
pub fn value_type_len(hive: *mut hive_h, value: u64) -> Result<(hive_type, u64)> {
    // Get type (u32)
    let v_type_ptr: *mut hive_type = &mut hive_type::hive_t_REG_NONE;
    let v_len_ptr: *mut u64 = &mut 0;
    let _ = unsafe { hivex_value_type(hive, value, v_type_ptr, v_len_ptr) };
    match v_type_ptr.is_null() && v_len_ptr.is_null() {
        true => Err(error::Error::hivex_error("hivex_value_type")),
        false => Ok(unsafe { (*v_type_ptr, *v_len_ptr) }),
    }
}

/// Wraps 'hivex_value_value' function, returns Ok if successful and an error on failure
/// * Todo, add error parsing from errno.
pub fn value_value(hive: *mut hive_h, value: u64) -> Result<Vec<i8>> {
    // Get type (u32)
    let (mut v_type, mut v_len) = value_type_len(hive, value)?;
    let res = unsafe { hivex_value_value(hive, value, &mut v_type, &mut v_len) };
    match res.is_null() {
        true => Err(error::Error::hivex_error("hivex_value_value")),
        false => {
            //
            let slice = unsafe { std::slice::from_raw_parts(res, v_len as usize) };
            Ok(slice.to_vec())
        }
    }
}

pub mod value {
    use super::*;
    pub fn string(hive: *mut hive_h, value: u64) -> Result<String> {
        let res = unsafe { hivex_value_string(hive, value) };
        let c_string = unsafe { ffi::CString::from_raw(res) };
        match c_string.into_string() {
            Ok(s) => Ok(s),
            Err(e) => Err(error::Error::ffi_error(e)),
        }
    }
    pub fn strings(hive: *mut hive_h, value: u64) -> Result<Vec<String>> {
        let res = unsafe { hivex_value_multiple_strings(hive, value) };
        // Calculate len of resulting array - There may very well be a nicer way to go about this but I cannot find anything atm.
        // Iterate over and find _, 0, 0, 0 to determine null terminated ascii strings. I'm assuming no values > 127 are expected here hence not doing smarter checks on null terminators.
        let data = value_value(hive, value)?;
        let mut n = 0;
        for (i, j) in (4..data.len()).enumerate() {
            if let [_, 0, 0, 0] = &data[i..j] {
                n += 1;
            }
        }
        let mut strings = Vec::new();
        let array: &[*mut i8] = unsafe { std::slice::from_raw_parts(res, n) };
        for i in array {
            let c_string = unsafe { ffi::CString::from_raw(*i) };
            match c_string.into_string() {
                Ok(s) => strings.push(s),
                Err(e) => return Err(error::Error::ffi_error(e)),
            }
        }
        Ok(strings)
    }
    pub fn dword(hive: *mut hive_h, value: u64) -> Result<i32> {
        let res = unsafe { hivex_value_dword(hive, value) };
        Ok(res)
    }
    pub fn qword(hive: *mut hive_h, value: u64) -> Result<i64> {
        let res = unsafe { hivex_value_qword(hive, value) };
        Ok(res)
    }
}

pub fn convert_windows_to_epoch(ticks: i64) -> Result<i64> {
    Ok(ticks / 10000000 - 11644473600)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hive_open() {
        let res = open("../test_data/SOFTWARE");
        // println!("{:?}", res);
        assert!(res.is_ok());
    }

    #[test]
    fn hive_close() {
        let hive: Result<*mut hive_h> = open("../test_data/SOFTWARE");
        // println!("{:?}", hive);
        assert!(hive.is_ok());
        assert!(close(hive.unwrap()).is_ok());
    }

    #[test]
    fn hive_root() {
        let hive: Result<*mut hive_h> = open("../test_data/SOFTWARE");
        assert!(hive.is_ok());
        let hive = hive.unwrap();
        let root = root(hive);
        // println!("{:?}", root);
        assert!(root.is_ok());
        assert!(close(hive).is_ok());
    }

    // Disabled until bug is understood
    #[test]
    fn hivex_last_modified() {
        let hive: Result<*mut hive_h> = open("../test_data/SOFTWARE");
        assert!(hive.is_ok());
        let hive = hive.unwrap();
        // println!("a {:?}", hive);
        let last_modified = last_modified(hive);
        // println!("b {:?}", last_modified);
        assert!(last_modified.is_ok());
        assert!(close(hive).is_ok());
    }

    #[test]
    fn hive_node_last_modified() {
        let hive: Result<*mut hive_h> = open("../test_data/SOFTWARE");
        assert!(hive.is_ok());
        let hive = hive.unwrap();
        let root = root(hive);
        assert!(root.is_ok());
        let root = root.unwrap();
        let lm = node_last_modified(hive, root);
        // println!("{:?}", lm);
        assert!(lm.is_ok());
        // println!("{:?}", convert_windows_to_epoch(lm.unwrap()));
        assert!(close(hive).is_ok());
    }

    #[test]
    fn hive_values() {
        let hive: Result<*mut hive_h> = open("../test_data/SOFTWARE");
        assert!(hive.is_ok());
        let hive = hive.unwrap();
        let root = root(hive);
        assert!(root.is_ok());
        let root = root.unwrap();

        let child = node_children(hive, root);
        println!("{:?}", child);
        assert!(child.is_ok());
        let child = child.unwrap()[0];

        let values = node_values(hive, child);
        println!("{:?}", values);
        assert!(close(hive).is_ok());
    }

    #[test]
    fn hive_value_type() {
        let hive: Result<*mut hive_h> = open("../test_data/SOFTWARE");
        assert!(hive.is_ok());
        let hive = hive.unwrap();
        let root = root(hive);
        assert!(root.is_ok());
        let root = root.unwrap();

        let child = node_children(hive, root);
        println!("{:?}", child);
        assert!(child.is_ok());
        let child = child.unwrap()[0];

        let values = node_values(hive, child);
        println!("{:?}", values);

        let v_type = value_type_len(hive, values.unwrap()[0]);
        println!("{:?}", v_type);

        assert!(close(hive).is_ok());
    }
}
