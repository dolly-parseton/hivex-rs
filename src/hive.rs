//
use chrono::{DateTime, Utc};
use hivex_sys::{hive_h, Result};
use std::{fmt, path};
//
#[derive(Debug)]
pub struct Hive {
    inner: *mut hive_h,
    nodes: Vec<u64>,
}

impl Hive {
    /// Create a new handle for a registry HIVE using a file path.
    pub fn new<P: AsRef<path::Path>>(path: P) -> Result<Self> {
        let mut hive = Self {
            inner: hivex_sys::open(path)?,
            nodes: Default::default(),
        };
        hive.recurse_nodes().map(|_| hive)
    }
    fn recurse_nodes(&mut self) -> Result<()> {
        // Iterate over the nodes and append the node value to the array.
        // Get root.
        let root = hivex_sys::root(self.inner)?;
        // Get node children recursively.
        Self::node_child_recurse(self.inner, root, &mut self.nodes)?;
        // Return when successful.
        Ok(())
    }
    fn node_child_recurse(hive: *mut hive_h, node: u64, nodes: &mut Vec<u64>) -> Result<()> {
        nodes.push(node);
        for child in hivex_sys::node_children(hive, node)?.iter() {
            Self::node_child_recurse(hive, *child, nodes)?;
        }
        Ok(())
    }
}
//
impl Iterator for Hive {
    type Item = Result<Node>;
    fn next(&mut self) -> Option<Self::Item> {
        self.nodes.pop().map(|n| Node::from_ptr(self.inner, n))
    }
}
//
#[derive(Debug)]
pub struct Node {
    name: String,
    path: String,
    last_modified: DateTime<Utc>,
    values: Vec<Value>,
}

impl Node {
    /// Using the hive pointer and node reference build a Node struct.
    pub fn from_ptr(hive: *mut hive_h, node: u64) -> Result<Self> {
        let mut values = Vec::new();
        for v in hivex_sys::node_values(hive, node)? {
            values.push(Value::from_ptr(hive, v)?);
        }
        Ok(Self {
            name: hivex_sys::node_name(hive, node)?,
            path: Self::construct_path(hive, node)?,
            last_modified: crate::epoch_to_timestamp(hivex_sys::convert_windows_to_epoch(
                hivex_sys::node_last_modified(hive, node)?,
            )?),
            values,
        })
    }
    /// Using the hive pointer and node reference recurse up the parent nodes and build the path of the node.
    fn construct_path(hive: *mut hive_h, node: u64) -> Result<String> {
        let mut path = String::new();
        let mut parts = Default::default();
        Self::recurse_node_parents(hive, node, &mut parts)?;
        parts.reverse();
        for part in parts {
            path.push_str("/");
            path.push_str(&part);
        }
        Ok(path)
    }
    fn recurse_node_parents(hive: *mut hive_h, node: u64, parts: &mut Vec<String>) -> Result<()> {
        match hivex_sys::node_name(hive, node)?.as_str() {
            "ROOT" => parts.push("ROOT".to_string()),
            n => {
                parts.push(n.to_string());
                if let Ok(n) = hivex_sys::node_parent(hive, node) {
                    Self::recurse_node_parents(hive, n, parts)?
                }
            }
        };
        Ok(())
    }
}
//
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Node: \"{}\"\n\tModified: \"{}\"\n\tPath: \"{}\"\n\tValues:",
            self.name, self.last_modified, self.path
        )?;
        for v in &self.values {
            write!(f, "\n\t\t(\"{}\", {:?}: {:?})", v.n, v.t, v.n)?;
        }
        write!(f, "\n")
    }
}
#[derive(Debug)]
pub struct Value {
    pub v: ValueValue,
    pub t: hivex_sys::hive_type,
    pub n: String,
}
impl Value {
    pub fn from_ptr(hive: *mut hive_h, value: u64) -> Result<Self> {
        let (t, _) = hivex_sys::value_type_len(hive, value)?;
        Ok(Self {
            v: ValueValue::from_ptr(hive, value)?,
            t,
            n: hivex_sys::value_key(hive, value)?,
        })
    }
}
//
#[derive(Debug)]
pub enum ValueValue {
    None,
    String(String),
    Strings(Vec<String>),
    DWord(i32),
    QWord(i64),
    Binary(Vec<i8>),
    Unsupported(Vec<i8>),
}

impl ValueValue {
    pub fn from_ptr(hive: *mut hive_h, value: u64) -> Result<Self> {
        // println!("{:?}", hivex_sys::value_value(hive, value)?);
        // println!("{:?}", hivex_sys::value_type_len(hive, value));
        match hivex_sys::value_type_len(hive, value) {
            Ok((hivex_sys::hive_type::hive_t_REG_NONE, _len)) => Ok(Self::None),
            Ok((hivex_sys::hive_type::hive_t_REG_SZ, _len)) => {
                Ok(Self::String(hivex_sys::value::string(hive, value)?))
            }
            Ok((hivex_sys::hive_type::hive_t_REG_EXPAND_SZ, _len)) => {
                Ok(Self::String(hivex_sys::value::string(hive, value)?))
            }
            Ok((hivex_sys::hive_type::hive_t_REG_LINK, _len)) => {
                Ok(Self::String(hivex_sys::value::string(hive, value)?))
            }
            Ok((hivex_sys::hive_type::hive_t_REG_BINARY, _len)) => {
                Ok(Self::Binary(hivex_sys::value_value(hive, value)?))
            }
            Ok((hivex_sys::hive_type::hive_t_REG_DWORD, _len)) => {
                Ok(Self::DWord(hivex_sys::value::dword(hive, value)?))
            }
            Ok((hivex_sys::hive_type::hive_t_REG_DWORD_BIG_ENDIAN, _len)) => {
                Ok(Self::DWord(hivex_sys::value::dword(hive, value)?))
            }
            Ok((hivex_sys::hive_type::hive_t_REG_QWORD, _len)) => {
                Ok(Self::QWord(hivex_sys::value::qword(hive, value)?))
            }
            Ok((hivex_sys::hive_type::hive_t_REG_MULTI_SZ, _len)) => {
                Ok(Self::Strings(hivex_sys::value::strings(hive, value)?))
            }
            Ok((_, _len)) => Ok(Self::Unsupported(hivex_sys::value_value(hive, value)?)),
            Err(e) => Err(e),
        }
    }
}
