use std::ops::Deref;
use std::path::{Component, Path};
use std::rc::Rc;

use heck::ToSnakeCase;
use nohash_hasher::IntMap;
use rustc_hash::FxHashMap;

pub struct Stree<T> {
    index: Vec<Rc<str>>,
    root: StreeNode<T>,
    last_id: StreeId,
    parents: IntMap<StreeId, StreeId>
}

pub struct StreeNode<T> {
    id: StreeId,
    key: Rc<str>,
    data: Option<T>,
    children: Vec<StreeNode<T>>,
    children_ids: IntMap<u32, u32>,
    children_keys: FxHashMap<Rc<str>, u32>,
}

// implement debug for stree:
impl<T: std::fmt::Debug> std::fmt::Debug for Stree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stree")
            .field("index", &self.index)
            .field("root", &self.root)
            .field("last_id", &self.last_id)
            .field("parents", &self.parents)
            .finish()
    }
}

impl<T: std::fmt::Debug> std::fmt::Debug for StreeNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreeNode")
            .field("key", &self.key)
            .field("id", &self.id)
            .field("data", &self.data)
            .field("children", &self.children)
            .field("children_ids", &self.children_ids)
            .field("children_keys", &self.children_keys)
            .finish()
    }
}

impl<T: Clone> Clone for Stree<T> {
    fn clone(&self) -> Self {
        Self {
            index: self.index.clone(),
            root: self.root.clone(),
            last_id: self.last_id,
            parents: self.parents.clone(),
        }
    }
}

impl<T: Clone> Clone for StreeNode<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            key: Rc::clone(&self.key),
            data: self.data.clone(),
            children: self.children.clone(),
            children_ids: self.children_ids.clone(),
            children_keys: self.children_keys.clone(),
        }
    }
}

impl<T: PartialEq> PartialEq for Stree<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index &&
        self.root == other.root
    }
}

impl<T: PartialEq> PartialEq for StreeNode<T> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key &&
        self.data == other.data &&
        self.children == other.children &&
        self.children_ids == other.children_ids &&
        self.children_keys == other.children_keys
    }
}

impl<T: Eq> Eq for Stree<T> {}
impl<T: Eq> Eq for StreeNode<T> {}

impl<T> Default for Stree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> StreeNode<T> {
    pub fn id(&self) -> StreeId { self.id }
    pub fn key(&self) -> &str { &self.key }
    pub fn data(&self) -> Option<&T> { self.data.as_ref() }
    pub fn data_mut(&mut self) -> Option<&mut T> { self.data.as_mut() }
    pub fn data_deref(&self) -> Option<&T::Target>
    where 
        T: Deref
    {
        self.data.as_deref()
    }
    
    pub fn children(&self) -> &Vec<StreeNode<T>> { &self.children }
    pub fn iter_children(&self) -> std::slice::Iter<'_, StreeNode<T>> { self.children.iter() }
    pub fn iter_children_mut(&mut self) -> std::slice::IterMut<'_, StreeNode<T>> { self.children.iter_mut() }
    pub fn iter(&self) -> StreeIter<'_, T> { StreeIter::new(self) }
    pub fn iter_mut(&mut self) -> StreeIterMut<'_, T> { StreeIterMut::new(self) }
    pub fn has_children(&self) -> bool { !self.children.is_empty() }
    pub fn has_data(&self) -> bool { self.data.is_some() }
    
    pub fn find<'a,'b>(&'a self, parts: &'b StreeKeys<'b>) -> Option<&'a StreeNode<T>> {
        let mut node = self;
        for part in &parts.parts {
            match node.children_keys.get(*part) {
                Some(index) => node = &node.children[*index as usize],
                None => return None,
            }
        }
        
        Some(node)
    }
    
    pub fn find_from<'a, P, PE>(&'a self, parts: P) -> Result<Option<&'a StreeNode<T>>, StreeError>
    where
        P: TryInto<StreeKeys<'a>, Error = PE>,
        PE: Into<StreeError>,
    {
        let parts: StreeKeys<'a> = parts.try_into().map_err(Into::into)?;
        Ok(self.find(&parts))
    }
    
    pub fn get(&self, node_id: StreeId) -> Option<&StreeNode<T>> {
        if self.id == node_id {
            return Some(self);
        }
        
        self.children_ids.get(&node_id).map(|idx| &self.children[(*idx) as usize])
    }
    
    pub fn get_mut(&mut self, node_id: StreeId) -> Option<&mut StreeNode<T>> {
        if self.id == node_id {
            return Some(self);
        }
        
        self.children_ids.get(&node_id).map(|idx| &mut self.children[(*idx) as usize])
    }
    
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreeKeys<'a> {
    parts: Vec<&'a str>,
}

#[derive(Debug, snafu::Snafu)]
pub enum StreeError {
    Utf8Path,
    Path,
    NotFound
}

impl<'a> StreeKeys<'a> {
    pub fn new(parts: Vec<&'a str>) -> Self {
        Self { parts }
    }
    
    pub fn parts(&self) -> &Vec<&str> { &self.parts }
    
    pub fn from_path(path: &'a Path) -> Result<Self, StreeError> {
        let mut parts = vec![];
        
        for c in path.components() {
            match c {
                Component::Prefix(prefix) => match prefix.as_os_str().to_str() {
                    Some(s) => parts.push(s),
                    None => return Err(StreeError::Utf8Path),
                },
                Component::RootDir => match parts.is_empty() {
                    true => {},
                    false => return Err(StreeError::Path),
                },
                Component::CurDir => {},
                Component::ParentDir => match parts.is_empty() {
                    false => { parts.pop(); },
                    true => return Err(StreeError::Path),
                },
                Component::Normal(os_str) => match os_str.to_str() {
                    Some(s) => parts.push(s),
                    None => return Err(StreeError::Utf8Path),
                },
            }
        }
        
        Ok(Self { parts })
    }
    
    pub fn join(&self, delimiter: &str) -> String {
        self.parts.join(delimiter)
    }
    
    pub fn to_snake_case(&self) -> String {
        self.parts.join("_").to_snake_case()
    }
}

impl<'a> TryFrom<&'a Path> for StreeKeys<'a> {
    type Error = StreeError;
    
    fn try_from(value: &'a Path) -> Result<Self, Self::Error> {
        Self::from_path(value)
    }
}

impl<T> Stree<T> {
    pub fn index(&self) -> &Vec<Rc<str>> { &self.index }
    pub fn root(&self) -> &StreeNode<T> { &self.root }
    
    pub fn new() -> Self {
        Self {
            index: Vec::new(),
            last_id: 0,
            parents: IntMap::default(),
            root: StreeNode {
                id: 0,
                key: Rc::from(""),
                data: None,
                children: vec![],
                children_ids: IntMap::default(),
                children_keys: FxHashMap::default(),
            },
        }
    }
    
    pub fn set<'a>(&mut self, parts: &StreeKeys<'a>, data: Option<T>) -> StreeId {
        let mut node = &mut self.root;
        
        for part in &parts.parts {
            let part = *part;
            if node.children_keys.contains_key(part) {
                node = &mut node.children[node.children_keys[part] as usize];
            } else {
                let key = match self.index.iter().find(|s| &***s == part) {
                    Some(key) => key.clone(),
                    None => {
                        let key: Rc<str> = Rc::from(part);
                        self.index.push(key.clone());
                        key
                    },
                };
                let hash_key = key.clone();
                self.last_id += 1;
                let child: StreeNode<T> = StreeNode {
                    key,
                    id: self.last_id,
                    data: None,
                    children: vec![],
                    children_ids: IntMap::default(),
                    children_keys: FxHashMap::default(),
                };
                
                node.children.push(child);
                let idx = (node.children.len() - 1) as u32;
                node.children_ids.insert(self.last_id, idx);
                node.children_keys.insert(hash_key, idx);
                self.parents.insert(self.last_id, node.id);
                node = &mut node.children[idx as usize]
            }
        }
        
        node.data = data;
        node.id
    }
    
    pub fn reserve_child_key<F>(&mut self, parent_node_id: StreeId, key: &str, data_fn: F) -> StreeSet
    where
        F: FnOnce() -> T,
    {
        let parent = self.get(parent_node_id);
        if parent.children_keys.contains_key(key) {
            return StreeSet::Existing(parent.children_keys[key]);
        }
        
        let key = match self.index.iter().find(|s| &***s == key) {
            Some(key) => key.clone(),
            None => {
                let key: Rc<str> = Rc::from(key);
                self.index.push(key.clone());
                key
            },
        };
        let hash_key = key.clone();
        self.last_id += 1;
        let id = self.last_id;
        let child: StreeNode<T> = StreeNode {
            key,
            id,
            data: Some(data_fn()),
            children: vec![],
            children_ids: IntMap::default(),
            children_keys: FxHashMap::default(),
        };
        
        self.parents.insert(id, parent_node_id);
        let parent = self.get_mut(parent_node_id);
        parent.children.push(child);
        let idx = (parent.children.len() - 1) as u32;
        parent.children_ids.insert(id, idx);
        parent.children_keys.insert(hash_key, idx);
        StreeSet::New(self.last_id)
    }
    
    pub fn append(&mut self, parent_node_id: StreeId, key: &str, data: Option<T>) -> StreeSet {
        let parent = self.get_mut(parent_node_id);
        if parent.children_keys.contains_key(key) {
            let child = &mut parent.children[parent.children_keys[key] as usize];
            child.data = data;
            return StreeSet::Existing(child.id);
        }
        
        let key = match self.index.iter().find(|s| &***s == key) {
            Some(key) => key.clone(),
            None => {
                let key: Rc<str> = Rc::from(key);
                self.index.push(key.clone());
                key
            },
        };
        let hash_key = key.clone();
        self.last_id += 1;
        let id = self.last_id;
        let child: StreeNode<T> = StreeNode {
            key,
            id,
            data,
            children: vec![],
            children_ids: IntMap::default(),
            children_keys: FxHashMap::default(),
        };
        
        self.parents.insert(id, parent_node_id);
        let parent = self.get_mut(parent_node_id);
        parent.children.push(child);
        let idx = (parent.children.len() - 1) as u32;
        parent.children_ids.insert(id, idx);
        parent.children_keys.insert(hash_key, idx);
        StreeSet::New(self.last_id)
    }
    
    pub fn set_from<'a, P, PE>(&mut self, parts: P, data: Option<T>) -> Result<StreeId, StreeError>
    where
        P: TryInto<StreeKeys<'a>, Error = PE>,
        PE: Into<StreeError>,
    {
        let parts: StreeKeys<'a> = parts.try_into().map_err(Into::into)?;
        Ok(self.set(&parts, data))
    }
    
    pub fn get(&self, node_id: StreeId) -> &StreeNode<T> {
        let path = self.path(node_id);
        self.get_by(&path)
    }
    
    pub fn get_by(&self, path: &StreePath) -> &StreeNode<T> {
        let mut node = &self.root;
        for id in &path.ids {
            node = &node.children[node.children_ids[id] as usize];
        }
        
        node
    }
    
    pub fn get_by_mut(&mut self, path: &StreePath) -> &mut StreeNode<T> {
        let mut node = &mut self.root;
        for id in &path.ids {
            node = &mut node.children[node.children_ids[id] as usize];
        }
        
        node
    }
    
    pub fn get_mut(&mut self, node_id: StreeId) -> &mut StreeNode<T> {
        let path = self.path(node_id);
        self.get_by_mut(&path)
    }
    
    pub fn parent_of(&self, node: &StreeNode<T>) -> &StreeNode<T> {
        let parent_id = self.parents.get(&node.id).expect("exists in tree");
        self.get(*parent_id)
    }
    
    pub fn find<'a,'b>(&'a self, parts: &'b StreeKeys<'b>) -> Option<&'a StreeNode<T>> {
        self.root.find(parts)
    }
    
    pub fn find_from<'a, P, PE>(&'a self, parts: P) -> Result<Option<&'a StreeNode<T>>, StreeError>
    where
        P: TryInto<StreeKeys<'a>, Error = PE>,
        PE: Into<StreeError>,
    {
        self.root.find_from(parts)
    }
    
    pub fn path_of(&self, node: &StreeNode<T>) -> StreePath {
        let mut ids = vec![];
        let mut current_node = node;
        
        loop {
            ids.push(current_node.id);
            match self.parents.get(&current_node.id) {
                Some(parent_id) => {
                    current_node = self.get(*parent_id);
                },
                None => break,
            }
        }
        
        ids.pop();
        ids.reverse();
        StreePath { ids }
    }
    
    pub fn path(&self, node_id: StreeId) -> StreePath {
        let mut ids = vec![node_id];
        let mut current_node_id = node_id;
        
        loop {
            match self.parents.get(&current_node_id) {
                Some(parent_id) => {
                    ids.push(*parent_id);
                    current_node_id = *parent_id;
                },
                None => break,
            }
        }
        
        ids.pop();
        ids.reverse();
        StreePath { ids }    
    }
    
    pub fn keys_of<'a>(&'a self, node: &'a StreeNode<T>) -> StreeKeys<'a> {
        let mut parts = vec![];
        let mut current_node = node;
        
        loop {
            parts.push(current_node.key());
            match self.parents.get(&current_node.id) {
                Some(parent_id) => {
                    current_node = self.get(*parent_id);
                },
                None => break,
            }
        }
        
        parts.pop();
        parts.reverse();
        StreeKeys { parts }
    }
    
    pub fn keys(&self, node_id: StreeId) -> StreeKeys<'_> {
        let node = self.get(node_id);
        self.keys_of(node)
    }
    
    pub fn parent_keys(&self, node_id: StreeId) -> StreeKeys<'_> {
        let parent_id = self.parents.get(&node_id).expect("exists in tree");
        self.keys(*parent_id)
    }
    
    pub fn to_path_string(&self, node_id: StreeId) -> String {
        self.keys(node_id).join("/")
    }
}

pub type StreeOk<O, T> = (O, Stree<T>);
pub type StreeErr<E, T> = (E, Stree<T>);
pub type StreeResult<O, T, E> = Result<StreeOk<O, T>, StreeErr<E, T>>;

pub type StreeId = u32;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreePath {
    ids: Vec<StreeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreeSet {
    New(StreeId),
    Existing(StreeId),
}

impl StreeSet {
    pub fn into_id(self) -> StreeId {
        match self {
            StreeSet::New(id) => id,
            StreeSet::Existing(id) => id,
        }
    }
    
    pub fn into_tuple(self) -> (StreeId, bool) {
        match self {
            StreeSet::New(id) => (id, true),
            StreeSet::Existing(id) => (id, false),
        }
    }
}

pub struct StreeIter<'a, T> {
    nodes: Vec<&'a StreeNode<T>>,
}

impl<'a, T> StreeIter<'a, T> {
    fn new(node: &'a StreeNode<T>) -> Self {
        Self {
            nodes: vec![node],
        }
    }
}

impl<'a, T> Iterator for StreeIter<'a, T> {
    type Item = &'a StreeNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.nodes.pop()?;
        self.nodes.extend(node.children.iter().rev());
        Some(node)
    }
}

pub struct StreeIterMut<'a, T> {
    nodes: Vec<&'a mut StreeNode<T>>,
}

impl<'a, T> StreeIterMut<'a, T> {
    fn new(node: &'a mut StreeNode<T>) -> Self {
        Self {
            nodes: vec![node],
        }
    }
}

impl<'a, T> Iterator for StreeIterMut<'a, T> {
    type Item = &'a mut StreeNode<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.nodes.pop()?;
        let children = &mut node.children as *mut Vec<StreeNode<T>>;
        // SAFETY:
        // - each node is yielded once
        // - no memory overlap on yielded node's children
        unsafe {
            self.nodes.extend((*children).iter_mut().rev());
        }
        Some(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test() {
        let mut stree: Stree<String> = Stree::new();
        
        let expected = Some(String::from("hello"));
        stree.set_from(Path::new("foo/bar"), expected.clone()).unwrap();
        dbg!(&stree);
        let actual = stree.find_from(Path::new("/foo/bar")).unwrap().and_then(StreeNode::data_deref);
        assert_eq!(expected.as_deref(), actual);
    }
    
    #[test]
    fn test_reserve() {
        let mut stree: Stree<String> = Stree::new();
        let foo_keys = StreeKeys::from_path(Path::new("foo")).unwrap();
        stree.set(&foo_keys, Some("FOO".to_string()));
        let foo_id = stree.find(&foo_keys).unwrap().id();
        let set = stree.reserve_child_key(foo_id, "bar", || "BAR".to_string());
        assert_eq!(StreeSet::New(2), set);
        let actual = stree.find_from(Path::new("/foo/bar")).unwrap().and_then(StreeNode::data_deref);
        assert_eq!(Some("BAR"), actual);
    }
}