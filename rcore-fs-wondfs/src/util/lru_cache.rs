extern crate alloc;
use spin::RwLock;
use alloc::sync::Arc;
use std::collections::HashMap;

pub struct LRUCache<T> {
    size: usize,
    capacity: usize,
    head: Link<T>,
    tail: Link<T>,
    map: HashMap<u32, Link<T>>,
}

type Link<T> = Option<Arc<RwLock<Node<T>>>>;

struct Node<T> {
    key: u32,
    elem: T,
    next: Link<T>,
    prev: Link<T>,
}

#[derive(Copy, Clone)]
struct NodeEntry<T> {
    key: u32,
    elem: T,
}

impl<T> Node<T> {
    fn new(entry: NodeEntry<T>) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Node {
            key: entry.key,
            elem: entry.elem,
            prev: None,
            next: None,
        }))
    }
}

impl <T: Copy> LRUCache<T> {
    pub fn new(capacity: usize) -> Self {
        LRUCache {
            capacity,
            size: 0,
            head: None, 
            tail: None,
            map: HashMap::with_capacity(capacity),
        }
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn contains_key(&self, key: u32) -> bool {
        self.map.contains_key(&key)
    }
    
    pub fn get(&mut self, key: u32) -> Option<T> {
        if !self.map.contains_key(&key) {
            return None;
        }
        let node = self.map.get(&key).unwrap();
        let node = node.as_ref().unwrap();
        let mut node = Some(Arc::clone(node));
        let entry = self.delete_node(&mut node);
        self.push_front(entry);
        let node = self.map[&key].as_ref().unwrap();
        Some(node.read().elem)
    }

    pub fn put(&mut self, key: u32, value: T) {
        if self.map.contains_key(&key) {
            let node = self.map.get(&key).unwrap();
            let node = node.as_ref().unwrap();
            let mut node = Some(Arc::clone(node));
            let mut entry = self.delete_node(&mut node);
            entry.elem = value;
            self.push_front(entry);
            return;
        }
        if self.size == self.capacity {
            let _ = self.pop_back();
        }
        let entry = NodeEntry {
            key,
            elem: value,
        };
        self.push_front(entry);
    }

    pub fn remove(&mut self, key: u32) {
        if self.map.contains_key(&key) {
            let node = self.map.get(&key).unwrap();
            let node = node.as_ref().unwrap();
            let mut node = Some(Arc::clone(node));
            self.delete_node(&mut node);
        }
    }
}

impl<T: Copy> LRUCache<T> {
    fn delete_node(&mut self, node: &mut Link<T>) -> NodeEntry<T> {
        let node = node.take().unwrap();
        let pre_node = node.write().prev.take();
        let next_node = node.write().next.take();
        let entry = NodeEntry {
            key: node.read().key,
            elem: node.read().elem,
        };
        self.map.remove(&entry.key);
        self.size -= 1;
        if pre_node.is_none() && next_node.is_none() {
            self.head.take();
            self.tail.take();
            return entry;
        }
        if pre_node.is_none() {
            let next_node = next_node.unwrap();
            next_node.write().prev.take();
            self.head = Some(Arc::clone(&next_node));
            return entry;
        }
        if next_node.is_none() {
            let pre_node = pre_node.unwrap();
            pre_node.write().next.take();
            self.tail = Some(Arc::clone(&pre_node));
            return entry;
        }
        let pre_node = pre_node.unwrap();
        let next_node = next_node.unwrap();
        pre_node.write().next = Some(Arc::clone(&next_node));
        next_node.write().prev = Some(Arc::clone(&pre_node));
        entry
    }

    fn push_front(&mut self, entry: NodeEntry<T>) {
        let new_head = Node::new(entry);
        match self.head.take() {
            Some(old_head) => {
                old_head.write().prev = Some(new_head.clone());
                new_head.write().next = Some(old_head);
                self.head = Some(new_head.clone());
            }
            None => {
                self.tail = Some(new_head.clone());
                self.head = Some(new_head.clone());
            }
        }
        self.size += 1;
        self.map.insert(entry.key,Some(new_head));
    }

    fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|old_tail| {
            match old_tail.write().prev.take() {
                Some(new_tail) => {
                    new_tail.write().next.take();
                    self.tail = Some(new_tail);
                }
                None => {
                    self.head.take();
                }
            }
            self.size -= 1;
            self.map.remove(&old_tail.write().key);
            Arc::try_unwrap(old_tail).ok().unwrap().into_inner().elem
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basics() {
        let mut lru = LRUCache::<u32>::new(5);
        lru.put(0,1);
        lru.put(1,2);
        assert_eq!(lru.get(0).unwrap(), 1);
        assert_eq!(lru.get(1).unwrap(), 2);
        lru.put(0,1);
        lru.put(1,2);
        lru.put(2,3);
        lru.put(3,4);
        lru.put(4,5);
        lru.put(5,6);
        assert_eq!(lru.get(3).unwrap(), 4);
        assert_eq!(lru.get(4).unwrap(), 5);
        assert_eq!(lru.get(5).unwrap(), 6);
        {
            lru.remove(5);
            let res = lru.get(5);
            assert!(res.is_none());
        }
        let res = lru.get(0);
        assert!(res.is_none());
    }
}