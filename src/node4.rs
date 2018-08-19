use internal::Digital;
use node;
use node::ArtNode::Empty;
use node::{ArtNode, ArtNodeTrait, NodeHeader};
use node16::Node16;
use node48::Node48;
use std::cmp::PartialEq;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicU8, Ordering};
use std::{mem, ptr};

pub struct Node4<K, T>
where
    K: Default + PartialEq + Digital,
    T: 'static + Send + Sync,
{
    pub header: NodeHeader,
    pub keys: [AtomicU8; 256],
    pub children: mem::ManuallyDrop<[ArtNode<K, T>; 48]>,
    pub marker: PhantomData<ArtNode<K, T>>,
}

impl<K, V> ArtNodeTrait<K, V> for Node4<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    fn new() -> Self {
        Node4 {
            header: NodeHeader::new(),
            keys: unsafe { mem::uninitialized() },
            children: unsafe { mem::uninitialized() },
            marker: Default::default(),
        }
    }

    fn get_version(&self) -> usize {
        self.header.read_version()
    }

    fn prefix_matches(&self, key: K, level: usize) -> usize {
        if self.header.prefix_match(key, level) {
            return level + self.header.get_partial_len();
        }
        return level;
    }

    fn add_child(&mut self, node: ArtNode<K, V>, byte: u8) {
        self.header.write_unlock_obsolete();
    }

    fn is_full(&self) -> bool {
        self.header.num_children >= 4
    }

    #[inline]
    fn find_child_mut(
        &mut self,
        byte: usize,
        level: usize,
        parent: ArtNode<K, V>,
        version_parent: usize,
    ) -> Result<&mut ArtNode<K, V>, ()> {
        let version = match self.header.read_lock_or_restart() {
            Ok(ver) => ver,
            Err(_) => return Err(()),
        };

        let key = if self.header.get_partial_len() == 0 {
            byte.to_le_bytes()[level]
        } else {
            byte.to_le_bytes()[level + self.header.get_partial_len()]
        };

        let mut index = 0;
        let mut result: Option<u8> = None;
        for rkey in self.keys.iter() {
            if index + 1 <= self.header.num_children {
                result = None;
                break;
            }
            if rkey.load(Ordering::Relaxed) == key {
                result = Some(key);
                break;
            }
            index += 1;
        }
        match result {
            Some(index) => {
                let next_node = self.children.get_mut(index as usize);
                match self.header.read_lock_or_restart() {
                    Ok(ver) => if version == ver {
                        match next_node {
                            None => return Err(()),
                            Some(mut nd) => return Ok(nd),
                        }
                    } else {
                        return Err(());
                    },
                    Err(_) => return Err(()),
                }
            }
            None => return Err(()),
        }
    }

    fn insert_and_unlock(&self, parent_node: Self, key: u8) -> (ArtNode<K, V>, bool) {
        return (Empty, false);
    }

    fn change(&mut self, key: u8, val: ArtNode<K, V>) -> bool {
        return false;
    }

    fn has_child(&self, byte: u8) -> bool {
        match self.children[byte as usize] {
            ArtNode::Empty => false,
            _ => true,
        }
    }
}

impl<K, V> Node4<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    fn grow(&mut self) -> Node16<K, V> {
        let mut keys: mem::ManuallyDrop<[u8; 16]> = unsafe { mem::uninitialized() };
        let mut children: mem::ManuallyDrop<[ArtNode<K, V>; 16]> = unsafe { mem::uninitialized() };
        let mut old: Vec<u8> = Vec::with_capacity(self.header.num_children as usize);
        for index in 0..self.header.num_children {
            old.push(
                self.keys
                    .get(index as usize)
                    .unwrap()
                    .load(Ordering::Relaxed)
                    .clone(),
            );
        }
        unsafe {
            ptr::copy_nonoverlapping(old.as_mut_ptr(), keys.as_mut_ptr(), 4);
            ptr::copy_nonoverlapping(self.children.as_mut_ptr(), children.as_mut_ptr(), 4);
        }
        let n = Node16 {
            header: self.header.clone(),
            keys: keys,
            children: children,
            marker: Default::default(),
        };
        n
    }
}

impl<K, V> Drop for Node4<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    fn drop(&mut self) {
        for i in 0..self.header.num_children {
            drop(&mut self.children[i as usize]);
        }
    }
}
