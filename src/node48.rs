use internal::Digital;
use node;
use node::ArtNode::Empty;
use node::{ArtNode, ArtNodeTrait, NodeHeader};
use node16::Node16;
use node256::Node256;
use std::cmp::PartialEq;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU8, Ordering};
use std::mem;
use std::mem::ManuallyDrop;

pub struct Node48<K, T>
where
    K: Default + PartialEq + Digital,
    T: 'static + Send + Sync,
{
    pub header: NodeHeader,
    pub keys: [AtomicU8; 256],
    pub children: mem::ManuallyDrop<[ArtNode<K, T>; 48]>,
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
            keys:  unsafe { make_array!(256, AtomicU8::new(0)) },
            children: unsafe { ManuallyDrop::new(make_array!(48, ArtNode::Empty)) },
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
    fn grow(&mut self) -> Node256<K, V> {
        let mut children = unsafe { make_array!(256, ArtNode::Empty) };
        for index in 0..255 {
            let cindex = self
                .keys
                .get(index as usize)
                .unwrap()
                .load(Ordering::Relaxed);
            if cindex != 0 {
                let mut _c = children.get_mut(index as usize).unwrap();
                _c = self.children.get_mut(cindex as usize).unwrap();
            }
        }
        return Node256 {
            header: self.header.clone(),
            children: children,
            marker: Default::default(),
        };
    }

    fn downgrade(&mut self) -> Node16<K, V> {
        let mut keys: mem::ManuallyDrop<[u8; 16]> = unsafe { mem::uninitialized() };
        let mut children: mem::ManuallyDrop<[ArtNode<K, V>; 16]> = unsafe { mem::uninitialized() };
        let mut new_children_index = 0;
        for index in 0..255 {
            let cindex = self
                .keys
                .get(index as usize)
                .unwrap()
                .load(Ordering::Relaxed);
            if cindex != 0 {
                keys.push(index as u8);

                let mut _c = children.get_mut(new_children_index as usize).unwrap();
                _c = self.children.get_mut(cindex as usize).unwrap();
                new_children_index += 1;
            }
        }
        return Node16 {
            header: self.header.clone(),
            keys: keys,
            children: children,
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
