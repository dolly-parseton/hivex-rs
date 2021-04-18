#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(dead_code)]

// Add in bindings.rs
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Uses
use chrono::{DateTime, NaiveDateTime, Utc};
use errno::errno;
use std::{error, ffi, path, ptr};

pub type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

/// General function to convert chars pointer to a string.
/// # Safety
///
/// Function can fail is chars pointer is null.
pub unsafe fn chars_to_string(
    chars: *mut ::std::os::raw::c_char,
    string: &mut String,
) -> Result<()> {
    let c_string = ffi::CString::from_raw(chars);
    *string = c_string.into_string()?;
    Ok(())
}

#[derive(Debug)]
pub struct HiveHandle {
    inner: *mut hive_h,
}

impl HiveHandle {
    /// Safely wraps `open_hive` function, returns and error if path is not able to be converted.
    pub fn open<P>(path: P) -> Result<Self>
    where
        P: AsRef<path::Path>,
    {
        let path_str = match path.as_ref().to_str() {
            Some(s) => ffi::CString::new(s)?,
            None => return Err("Unable to convert path to string.".into()),
        };
        let mut handle = None;
        unsafe { open_hive(path_str.as_ptr(), &mut handle) };
        match handle {
            Some(h) => Ok(h),
            None => Err(errno().into()),
        }
    }
    /// Get the root node of this registry hive.
    pub fn root(&self) -> Result<NodeHandle> {
        match unsafe { root_hive(self.inner) } {
            0 => Err(errno().into()),
            n => Ok(NodeHandle { inner: n }),
        }
    }
    /// Get the last modified timestamp registry hive.
    pub fn last_modified(&self) -> DateTime<Utc> {
        let ticks = unsafe { last_modified_hive(self.inner) } / 10000000 - 11644473600;
        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(ticks, 0), Utc)
    }
}

/// Wraps 'hivex_open' function, function is unsafe
///
/// # Safety
///
/// Function will fail on certain registry hives.
pub unsafe fn open_hive(s: *const ::std::os::raw::c_char, handle: &mut Option<HiveHandle>) {
    let inner = hivex_open(s, 0);
    if !inner.is_null() {
        *handle = Some(HiveHandle { inner });
    }
}

/// Wraps 'hivex_root' function, returns 0 on error or the node_h on success
/// # Safety
///
/// Function will only work when the hive opened is valid.
pub unsafe fn root_hive(hive: *mut hive_h) -> hive_node_h {
    hivex_root(hive)
}

/// Wraps 'hivex_last_modified' function, returns an integer (no error handling)
/// # Safety
///
/// Fairly safe, no fail condition for this function in hivex.
pub unsafe fn last_modified_hive(hive: *mut hive_h) -> i64 {
    hivex_last_modified(hive)
}

#[derive(Debug, Clone)]
pub struct NodeHandle {
    inner: hive_node_h,
}

impl NodeHandle {
    /// Returns the name of the node. Can fail on either retrieving chars or converting CString.
    pub fn name(&self, hive_handle: &HiveHandle) -> Result<String> {
        let mut name = String::new();
        unsafe { name_node(hive_handle.inner, self.inner, &mut name)? };
        match !name.is_empty() {
            true => Ok(name),
            false => Err("Unable to get name of node.".into()),
        }
    }
    /// Returns the last modified timestamp of the node.
    pub fn last_modified(&self, hive_handle: &HiveHandle) -> DateTime<Utc> {
        let ticks =
            unsafe { last_modified_node(hive_handle.inner, self.inner) } / 10000000 - 11644473600;
        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(ticks, 0), Utc)
    }
    /// Get a vec containing all the child nodes. If there are no child nodes the vec is empty.
    pub fn children(&self, hive_handle: &HiveHandle) -> Result<Vec<NodeHandle>> {
        let mut children = Vec::new();
        unsafe { children_node(hive_handle.inner, self.inner, &mut children)? };
        Ok(children.iter().map(|c| NodeHandle { inner: *c }).collect())
    }
    /// Get a vec containing all the values handles. If there are no values handles the vec is empty.
    pub fn values(&self, hive_handle: &HiveHandle) -> Result<Vec<ValueHandle>> {
        let mut values = Vec::new();
        unsafe { values_node(hive_handle.inner, self.inner, &mut values)? };
        Ok(values.iter().map(|v| ValueHandle { inner: *v }).collect())
    }
    /// Returns the parent node, if the parent_node function fails then the result is None.
    pub fn parent(&self, hive_handle: &HiveHandle) -> Option<NodeHandle> {
        let mut parent = 0;
        match unsafe { parent_node(hive_handle.inner, self.inner, &mut parent) } {
            Ok(()) => Some(NodeHandle { inner: parent }),
            Err(_) => None,
        }
    }
}

/// Wraps 'hivex_node_name' function, returns null on failure and sets errno
/// # Safety
///
/// Should not be called on an invalid hive or node value.
pub unsafe fn name_node(hive: *mut hive_h, node: hive_node_h, name: &mut String) -> Result<()> {
    let chars = hivex_node_name(hive, node);
    if chars.is_null() {
        return Err(errno().into());
    }
    chars_to_string(chars, name)
}

/// Wraps 'hivex_node_timestamp' function, returns Ok if successful.
/// # Safety
///
/// Fairly safe, no fail condition for this function in hivex but if called on an bad node will likely fail.
pub unsafe fn last_modified_node(hive: *mut hive_h, node: hive_node_h) -> i64 {
    hivex_node_timestamp(hive, node)
}

/// Wraps 'hivex_node_children' and 'hivex_node_nr_children' function, returns an empty Result on success and errno on failure
pub unsafe fn children_node(
    hive: *mut hive_h,
    node: hive_node_h,
    children: &mut Vec<hive_node_h>,
) -> Result<()> {
    let res = hivex_node_children(hive, node);
    let n = hivex_node_nr_children(hive, node);
    if res.is_null() {
        return Err(errno().into());
    }
    *children = std::slice::from_raw_parts(res, n as usize).to_vec();
    Ok(())
}

/// Wraps 'hivex_node_values' and 'hivex_node_nr_values' function, returns an empty Result on success and errno on failure
/// # Safety
///
/// Will likely fail on an invalid hive or node.
pub unsafe fn values_node(
    hive: *mut hive_h,
    node: hive_node_h,
    values: &mut Vec<hive_value_h>,
) -> Result<()> {
    let res = hivex_node_values(hive, node);
    let n = hivex_node_nr_values(hive, node);
    if res.is_null() {
        return Err(errno().into());
    }
    *values = std::slice::from_raw_parts(res, n as usize).to_vec();
    Ok(())
}

pub unsafe fn parent_node(
    hive: *mut hive_h,
    node: hive_node_h,
    parent: &mut hive_node_h,
) -> Result<()> {
    *parent = hivex_node_parent(hive, node);
    match parent {
        0 => Err(errno().into()),
        _ => Ok(()),
    }
}

#[derive(Debug, Clone)]
pub struct ValueHandle {
    inner: hive_value_h,
}

impl ValueHandle {
    pub fn key(&self, hive_handle: &HiveHandle) -> Result<String> {
        let mut key = String::new();
        unsafe { key_value(hive_handle.inner, self.inner, &mut key)? };
        if key == "" {
            key = String::from("(Default)")
        }
        Ok(key)
    }
    pub fn kind(&self, hive_handle: &HiveHandle) -> Result<hive_type> {
        let (t, _) = unsafe { meta_value(hive_handle.inner, self.inner)? };
        Ok(t)
    }
    pub fn len(&self, hive_handle: &HiveHandle) -> Result<usize> {
        let (_, l) = unsafe { meta_value(hive_handle.inner, self.inner)? };
        Ok(l)
    }
    pub fn meta(&self, hive_handle: &HiveHandle) -> Result<(hive_type, usize)> {
        let (t, l) = unsafe { meta_value(hive_handle.inner, self.inner)? };
        Ok((t, l))
    }
    pub fn raw(&self, hive_handle: &HiveHandle) -> Result<Vec<i8>> {
        let mut raw = Vec::new();
        unsafe { raw_value(hive_handle.inner, self.inner, &mut raw)? };
        Ok(raw)
    }
    pub fn string(&self, hive_handle: &HiveHandle) -> Result<String> {
        let mut string = String::new();
        unsafe { string_value(hive_handle.inner, self.inner, &mut string)? };
        Ok(string)
    }
    pub fn strings(&self, hive_handle: &HiveHandle) -> Result<Vec<String>> {
        let mut strings = Vec::new();
        unsafe { strings_value(hive_handle.inner, self.inner, &mut strings)? };
        Ok(strings)
    }
    pub fn dword(&self, hive_handle: &HiveHandle) -> Result<i32> {
        let mut dword = Default::default();
        unsafe { dword_value(hive_handle.inner, self.inner, &mut dword)? };
        Ok(dword)
    }
    pub fn qword(&self, hive_handle: &HiveHandle) -> Result<i64> {
        let mut qword = Default::default();
        unsafe { qword_value(hive_handle.inner, self.inner, &mut qword)? };
        Ok(qword)
    }
}

pub unsafe fn key_value(hive: *mut hive_h, value: hive_value_h, key: &mut String) -> Result<()> {
    let chars = hivex_value_key(hive, value);
    if chars.is_null() {
        return Err(errno().into());
    }
    chars_to_string(chars, key)
}

pub unsafe fn meta_value(hive: *mut hive_h, value: hive_value_h) -> Result<(hive_type, usize)> {
    let t: *mut hive_type = &mut hive_type::hive_t_REG_NONE;
    let l: *mut u64 = &mut 0;
    let res = hivex_value_type(hive, value, t, l);
    match (res, t.is_null(), l.is_null()) {
        (0, false, false) => Ok((*t, *l as usize)),
        _ => Err(errno().into()),
    }
}

pub unsafe fn raw_value(hive: *mut hive_h, value: hive_value_h, raw: &mut Vec<i8>) -> Result<()> {
    let t: *mut hive_type = &mut hive_type::hive_t_REG_NONE;
    let l: *mut u64 = &mut 0;
    let res = hivex_value_type(hive, value, t, l);
    match (res, t.is_null(), l.is_null()) {
        (0, false, false) => {
            let res = hivex_value_value(hive, value, t, l);
            if res.is_null() {
                return Err(errno().into());
            }
            *raw = std::slice::from_raw_parts(res, *l as usize).to_vec();
            Ok(())
        }
        _ => Err(errno().into()),
    }
}

pub unsafe fn string_value(
    hive: *mut hive_h,
    value: hive_value_h,
    string: &mut String,
) -> Result<()> {
    let chars = hivex_value_string(hive, value);
    if chars.is_null() {
        return Err(errno().into());
    }
    chars_to_string(chars, string)
}

pub unsafe fn strings_value(
    hive: *mut hive_h,
    value: hive_value_h,
    strings: &mut Vec<String>,
) -> Result<()> {
    let res = hivex_value_multiple_strings(hive, value);
    if res.is_null() {
        return Err(errno().into());
    }
    let mut raw = Vec::new();
    raw_value(hive, value, &mut raw)?;
    let mut n = 0;
    for (i, j) in (4..raw.len()).enumerate() {
        if let [_, 0, 0, 0] = &raw[i..j] {
            n += 1;
        }
    }
    for i in std::slice::from_raw_parts(res, n) {
        let mut string = String::new();
        chars_to_string(*i, &mut string)?;
        strings.push(string);
    }
    Ok(())
}

pub unsafe fn dword_value(hive: *mut hive_h, value: hive_value_h, dword: &mut i32) -> Result<()> {
    *dword = hivex_value_dword(hive, value);
    Ok(())
}

pub unsafe fn qword_value(hive: *mut hive_h, value: hive_value_h, qword: &mut i64) -> Result<()> {
    *qword = hivex_value_qword(hive, value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hive_open() {
        let hive_handle = HiveHandle::open("../.test_data/SOFTWARE");
        println!("{:?}", hive_handle);
        assert!(hive_handle.is_ok());
    }

    #[test]
    fn hive_close() {
        let hive_handle = HiveHandle::open("../.test_data/SOFTWARE");
        println!("{:?}", hive_handle);
        assert!(hive_handle.is_ok());
        drop(hive_handle.unwrap());
    }

    #[test]
    fn hive_root() {
        let hive_handle = HiveHandle::open("../.test_data/SOFTWARE");
        assert!(hive_handle.is_ok());
        let root_handle = hive_handle.unwrap().root();
        println!("{:?}", root_handle);
        assert!(root_handle.is_ok());
    }

    #[test]
    fn hivex_last_modified() {
        let hive_handle = HiveHandle::open("../.test_data/SOFTWARE");
        assert!(hive_handle.is_ok());
        let ts = hive_handle.unwrap().last_modified();
        println!("{:?}", ts);
    }

    #[test]
    fn hive_node_last_modified() -> Result<()> {
        let hive_handle = HiveHandle::open("../.test_data/SOFTWARE")?;
        let root_handle = hive_handle.root()?;
        println!("{:?}", root_handle);
        let ts = root_handle.last_modified(hive_handle);
        println!("{:?}", ts);
        Ok(())
    }

    #[test]
    fn hive_node_name() -> Result<()> {
        let hive_handle = HiveHandle::open("../.test_data/SOFTWARE")?;
        let root_handle = hive_handle.root()?;
        println!("{:?}", root_handle);
        let name = root_handle.name(hive_handle);
        println!("{:?}", name);
        Ok(())
    }

    #[test]
    fn hive_node_children() -> Result<()> {
        let hive_handle = HiveHandle::open("../.test_data/SOFTWARE")?;
        let root_handle = hive_handle.root()?;
        println!("{:?}", root_handle);
        let children = root_handle.children(hive_handle);
        println!("{:?}", children);
        Ok(())
    }

    // #[test]
    // fn hive_values() {
    //     let hive: Result<*mut hive_h> = open("../test_data/SOFTWARE");
    //     assert!(hive.is_ok());
    //     let hive = hive.unwrap();
    //     let root = root(hive);
    //     assert!(root.is_ok());
    //     let root = root.unwrap();

    //     let child = node_children(hive, root);
    //     println!("{:?}", child);
    //     assert!(child.is_ok());
    //     let child = child.unwrap()[0];

    //     let values = node_values(hive, child);
    //     println!("{:?}", values);
    //     assert!(close(hive).is_ok());
    // }

    // #[test]
    // fn hive_value_type() {
    //     let hive: Result<*mut hive_h> = open("../test_data/SOFTWARE");
    //     assert!(hive.is_ok());
    //     let hive = hive.unwrap();
    //     let root = root(hive);
    //     assert!(root.is_ok());
    //     let root = root.unwrap();

    //     let child = node_children(hive, root);
    //     println!("{:?}", child);
    //     assert!(child.is_ok());
    //     let child = child.unwrap()[0];

    //     let values = node_values(hive, child);
    //     println!("{:?}", values);

    //     let v_type = value_type_len(hive, values.unwrap()[0]);
    //     println!("{:?}", v_type);

    //     assert!(close(hive).is_ok());
    // }
}
