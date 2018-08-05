use internal::Digital;
use node;
use node::ArtNode::Empty;
use node::{ArtNode, ArtNodeTrait, NodeHeader};
use node48::Node48;
use std::cmp::PartialEq;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicU8, Ordering};

pub struct Node256<K, T>
where
    K: Default + PartialEq + Digital,
    T: 'static + Send + Sync,
{
    pub header: NodeHeader,
    pub children: [ArtNode<K, T>; 256],
    pub marker: PhantomData<T>,
}

impl<K, V> ArtNodeTrait<K, V> for Node256<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    fn new() -> Self {
        Node256 {
            header: NodeHeader::new(),
            children: unsafe { make_array!(256, ArtNode::Empty) },
            marker: Default::default(),
        }
    }

    fn add_child(&mut self, node: ArtNode<K, V>, byte: u8) {}

    fn is_full(&self) -> bool {
        self.header.num_children >= 256
    }

    #[inline]
    fn find_child_mut(
        &mut self,
        byte: usize,
        level: usize,
        parent: ArtNode<K, V>,
        version_parent: usize,
    ) -> Result<&mut ArtNode<K, V>, bool> {
        let mut version = 0;
        loop {
            match self.header.read_lock_or_restart() {
                Ok(ver) => {
                    version = ver;
                    break;
                }
                Err(true) => continue,
                Err(false) => return Err((false)),
            }
        }
        let index = if self.header.get_partial_len() == 0 {
            byte.to_le().to_bytes()[level]
        } else {
            byte.to_le().to_bytes()[level + self.header.get_partial_len()]
        };

        let next_node = self.children.get_mut(index as usize);
        loop {
            match self.header.read_lock_or_restart() {
                Ok(ver) => if version == ver {
                    match next_node {
                        None => return Err(true),
                        Some(mut nd) => return Ok(nd),
                    }
                } else {
                    return Err(true);
                },
                Err(true) => continue,
                Err(false) => return Err(false),
            }
        }
    }

    fn insertAndUnlock(&self, parent_node: Self, key: u8) -> (ArtNode<K, V>, bool) {
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

impl<K, V> Node256<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    fn shrink(&mut self) -> Node48<K, V> {
        let mut keys = unsafe { make_array!(256, AtomicU8::new(0)) };
        let mut children = unsafe { ManuallyDrop::new(make_array!(48, ArtNode::Empty)) };
        let mut new_children_index = 0;
        for index in 0..255 {
            match self.children.get(index as usize).unwrap() {
                ArtNode::Empty => continue,
                _ => {
                    let mut _k = keys.get_mut(index).unwrap();
                    _k.store(new_children_index as u8, Ordering::Relaxed);

                    let mut _c = children.get_mut(new_children_index as usize).unwrap();
                    _c = self.children.get_mut(index as usize).unwrap();
                    new_children_index += new_children_index;
                }
            }
        }
        return Node48 {
            header: NodeHeader::new(),
            keys: keys,
            children: children,
            marker: Default::default(),
        };
    }
}

impl<K, V> Drop for Node256<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    fn drop(&mut self) {
        for i in 0..256 {
            drop(&mut self.children[i as usize]);
        }
    }
}
