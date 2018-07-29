use internal::Digital;
use node;
use node::ArtNode::Empty;
use node::{ArtNode, ArtNodeTrait, NodeHeader};
use node16::Node16;
use node48::Node48;
use std::cmp::PartialEq;
use std::marker::PhantomData;
use std::mem;
use std::sync::atomic::{AtomicU8, Ordering};

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
    ) -> Result<&mut ArtNode<K, V>, bool> {
        let mut version = 0;
        loop {
            match self.header.read_lock_or_restart() {
                Ok(ver) => version = ver,
                Err(true) => continue,
                Err(false) => return Err((false)),
            }
        }

        let key = if self.header.get_partial_len() == 0 {
            byte.to_le().to_bytes()[level]
        } else {
            byte.to_le().to_bytes()[level + self.header.get_partial_len()]
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
            None => return Err(false),
        }
    }

    fn insertAndUnlock(&self, parent_node: Self, key: u8) -> (ArtNode<K, V>, bool) {
        return (Empty, false);
    }

    fn change(&mut self, key: u8, val: ArtNode<K, V>) -> bool {
        return false;
    }

    fn grow(&self) -> Option<ArtNode<K, V>> {
        return Some(ArtNode::Inner16(Node16::new()));
    }
}

impl<K, V> Node4<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    fn grow(&mut self) -> Node16<K, V> {
        let mut keys: Vec<u8> = Vec::with_capacity(16);
        let mut children = rep_no_copy!(ArtNode<K, V>; ArtNode::Empty; 16);
        for index in 0..self.header.num_children {
            keys.push(
                self.keys
                    .get(index as usize)
                    .unwrap()
                    .load(Ordering::Relaxed)
                    .clone(),
            );
            let mut _c = children.get_mut(index as usize).unwrap();
            _c = self.children.get_mut(index as usize).unwrap();
        }
        let mut n = Node16 {
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
