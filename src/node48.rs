use internal::Digital;
use node;
use node::ArtNode::Empty;
use node::{ArtNode, ArtNodeTrait, NodeHeader};
use node256::Node256;
use std::cmp::PartialEq;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU8, Ordering};

pub struct Node48<K, T>
where
    K: Default + PartialEq + Digital,
    T: 'static + Send + Sync,
{
    pub header: NodeHeader,
    pub keys: Vec<AtomicU8>,
    pub children: Vec<ArtNode<K, T>>,
    pub marker: PhantomData<T>,
}

impl<K, V> ArtNodeTrait<K, V> for Node48<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    fn new() -> Self {
        Node48 {
            header: NodeHeader::new(),
            keys: rep_no_copy!(AtomicU8; AtomicU8::new(0); 256),
            children: rep_no_copy!(ArtNode<K, V>; ArtNode::Empty; 48),
            marker: Default::default(),
        }
    }

    fn add_child(&mut self, node: ArtNode<K, V>, byte: u8) {}

    fn is_full(&self) -> bool {
        self.header.num_children >= 48
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
        let key = if self.header.get_partial_len() == 0 {
            byte.to_le().to_bytes()[level]
        } else {
            byte.to_le().to_bytes()[level + self.header.get_partial_len()]
        };
        let result = self.keys.get(key as usize);
        match result {
            Some(index) => {
                let id = index.load(Ordering::Relaxed);
                if id == 0 {
                    return Err(false);
                }
                let next_node = self.children.get_mut(id as usize);
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
        return Some(ArtNode::Inner256(Box::new(Node256::new())));
    }
}

impl<K, V> Node48<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    fn grow(&self) -> Node256<K, V> {
        return Node256 {
            header: self.header.clone(),
            children: rep_no_copy!(ArtNode<K, V>; ArtNode::Empty;  256),
            marker: Default::default(),
        };
    }
}

impl<K, V> Drop for Node48<K, V>
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
