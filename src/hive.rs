//
use hivex_sys::{hive_h, Result};
use std::path;
//
pub struct Hive {
    inner: *mut hive_h,
}

impl Hive {
    /// Create a new handle for a registry HIVE using a file path.
    pub fn new<P: AsRef<path::Path>>(path: P) -> Result<Self> {
        Ok(Self {
            inner: hivex_sys::open(path)?,
        })
    }
    ///
    pub fn get_node() {}
}
//
pub struct HiveIterator {
    hive: *mut hive_h,
    nodes: Vec<u64>,
}

impl HiveIterator {
    fn new(hive: *mut hive_h) -> Result<Self> {
        let mut iter = Self {
            hive,
            nodes: Default::default(),
        };
        match iter.populate_nodes() {
            Ok(()) => Ok(iter),
            Err(e) => Err(e),
        }
    }
    fn populate_nodes(&mut self) -> Result<()> {
        // Iterate over the nodes and append the node value to the array.
        Ok(())
    }
}
//
pub struct Node {
    // Name
// Values
}
//
pub struct Value {
    //
}
