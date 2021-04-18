//
use chrono::{DateTime, Utc};
use hivex_sys::Result;
use std::{fmt, path};
//
#[derive(Debug)]
pub struct Hive {
    handle: hivex_sys::HiveHandle,
    nodes: Vec<hivex_sys::NodeHandle>,
}

impl Hive {
    /// Create a new handle for a registry HIVE using a file path.
    pub fn new<P: AsRef<path::Path>>(path: P) -> Result<Self> {
        let mut hive = Self {
            handle: hivex_sys::HiveHandle::open(path)?,
            nodes: Vec::new(),
        };
        hive.recurse_nodes().map(|_| hive)
    }
    fn recurse_nodes(&mut self) -> Result<()> {
        // Iterate over the nodes and append the node value to the array.
        let root = self.handle.root()?;
        // Get node children recursively.
        Self::node_child_recurse(&self.handle, &root, &mut self.nodes)?;
        // Return when successful.
        Ok(())
    }
    fn node_child_recurse(
        hive: &hivex_sys::HiveHandle,
        node: &hivex_sys::NodeHandle,
        nodes: &mut Vec<hivex_sys::NodeHandle>,
    ) -> Result<()> {
        nodes.push(node.clone());
        for child in node.children(&hive)?.iter() {
            Self::node_child_recurse(&hive, child, nodes)?;
        }
        Ok(())
    }
}
//
impl Iterator for Hive {
    type Item = Result<Node>;
    fn next(&mut self) -> Option<Self::Item> {
        self.nodes
            .pop()
            .map(|n| Node::from_handle(&self.handle, &n))
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

#[derive(Debug)]
pub struct NodeByValue {
    name: String,
    path: String,
    last_modified: DateTime<Utc>,
    values: Value,
}

impl Node {
    /// Using the hive pointer and node reference build a Node struct.
    pub fn from_handle(hive: &hivex_sys::HiveHandle, node: &hivex_sys::NodeHandle) -> Result<Self> {
        let mut values = Vec::new();
        for v in node.values(&hive)?.iter() {
            values.push(Value::from_handle(hive, &v)?);
        }
        Ok(Self {
            name: node.name(hive)?,
            path: Self::construct_path(hive, node)?,
            last_modified: node.last_modified(hive),
            values,
        })
    }
    pub fn get_nodes_by_value(&self) -> Vec<NodeByValue> {
        let mut nodes = Vec::new();
        for v in &self.values {
            nodes.push(NodeByValue {
                name: self.name.clone(),
                path: self.path.clone(),
                last_modified: self.last_modified.clone(),
                values: v.clone(),
            });
        }
        nodes
    }
    /// Using the hive pointer and node reference recurse up the parent nodes and build the path of the node.
    fn construct_path(
        hive: &hivex_sys::HiveHandle,
        node: &hivex_sys::NodeHandle,
    ) -> Result<String> {
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
    fn recurse_node_parents(
        hive: &hivex_sys::HiveHandle,
        node: &hivex_sys::NodeHandle,
        parts: &mut Vec<String>,
    ) -> Result<()> {
        match node.name(hive)?.as_str() {
            "ROOT" => parts.push("ROOT".to_string()),
            n => {
                parts.push(n.to_string());
                if let Some(n) = node.parent(hive) {
                    Self::recurse_node_parents(hive, &n, parts)?
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
            write!(f, "\n\t\t(\"{}\", {:?}: {:?})", v.n, v.t, v.v)?;
        }
        write!(f, "\n")
    }
}
//
#[derive(Debug, Clone)]
pub struct Value {
    pub v: ValueValue,
    pub t: hivex_sys::hive_type,
    pub n: String,
    pub l: usize,
}
impl Value {
    pub fn from_handle(
        hive: &hivex_sys::HiveHandle,
        value: &hivex_sys::ValueHandle,
    ) -> Result<Self> {
        let (t, l) = value.meta(hive)?;
        Ok(Self {
            v: ValueValue::from_handle(hive, value)?,
            t,
            n: value.key(hive)?,
            l,
        })
    }
}
//
#[derive(Debug, Clone)]
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
    pub fn from_handle(
        hive: &hivex_sys::HiveHandle,
        value: &hivex_sys::ValueHandle,
    ) -> Result<Self> {
        match value.kind(hive)? {
            hivex_sys::hive_type::hive_t_REG_NONE => Ok(Self::None),
            hivex_sys::hive_type::hive_t_REG_SZ => Ok(Self::String(value.string(hive)?)),
            hivex_sys::hive_type::hive_t_REG_EXPAND_SZ => Ok(Self::String(value.string(hive)?)),
            hivex_sys::hive_type::hive_t_REG_LINK => Ok(Self::String(value.string(hive)?)),
            hivex_sys::hive_type::hive_t_REG_BINARY => Ok(Self::Binary(value.raw(hive)?)),
            hivex_sys::hive_type::hive_t_REG_DWORD => Ok(Self::DWord(value.dword(hive)?)),
            hivex_sys::hive_type::hive_t_REG_DWORD_BIG_ENDIAN => {
                Ok(Self::DWord(value.dword(hive)?))
            }
            hivex_sys::hive_type::hive_t_REG_QWORD => Ok(Self::QWord(value.qword(hive)?)),
            hivex_sys::hive_type::hive_t_REG_MULTI_SZ => Ok(Self::Strings(value.strings(hive)?)),
            _ => Ok(Self::Unsupported(value.raw(hive)?)),
        }
    }
}
