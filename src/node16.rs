use internal::Digital;
use node;
use node::ArtNode::Empty;
use node::{ArtNode, ArtNodeTrait, NodeHeader};
use node4::Node4;
use node48::Node48;
use std::arch::x86_64::__m128i;
use std::arch::x86_64::_mm_cmpeq_epi8;
use std::arch::x86_64::_mm_loadu_si128;
use std::arch::x86_64::_mm_movemask_epi8;
use std::arch::x86_64::_mm_set1_epi8;
use std::cmp::PartialEq;
use std::intrinsics::cttz;
use std::marker::PhantomData;
use std::mem;
use std::mem::ManuallyDrop;
use std::sync::atomic::{AtomicU8, Ordering};

pub struct Node16<K, T>
where
    K: Default + PartialEq + Digital,
    T: 'static + Send + Sync,
{
    pub header: NodeHeader,
    pub keys: mem::ManuallyDrop<[u8; 16]>,
    pub children: mem::ManuallyDrop<[ArtNode<K, T>; 16]>,
    pub marker: PhantomData<K>,
}

impl<K, V> ArtNodeTrait<K, V> for Node16<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    fn new() -> Self {
        Node16 {
            header: NodeHeader::new(),
            keys: unsafe { mem::uninitialized() },
            children: unsafe { mem::uninitialized() },
            marker: Default::default(),
        }
    }

    fn get_version(&self) -> usize {
        self.header.read_version()
    }

    fn prefix_matches(&self, key: K, level: usize) -> Result<usize, usize> {
        if self.header.prefix_match(key, level) {
            return Ok(level + self.header.get_partial_len());
        }
        return Err(level);
    }

    fn add_child(&mut self, node: ArtNode<K, V>, byte: u8) {}

    fn is_full(&self) -> bool {
        self.header.num_children >= 16
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
        let index = 0;

        let raw_node_key = [
            *self.keys.get(0).unwrap() as i8,
            *self.keys.get(1).unwrap() as i8,
            *self.keys.get(2).unwrap() as i8,
            *self.keys.get(3).unwrap() as i8,
            *self.keys.get(4).unwrap() as i8,
            *self.keys.get(5).unwrap() as i8,
            *self.keys.get(6).unwrap() as i8,
            *self.keys.get(7).unwrap() as i8,
            *self.keys.get(8).unwrap() as i8,
            *self.keys.get(9).unwrap() as i8,
            *self.keys.get(10).unwrap() as i8,
            *self.keys.get(11).unwrap() as i8,
            *self.keys.get(12).unwrap() as i8,
            *self.keys.get(13).unwrap() as i8,
            *self.keys.get(14).unwrap() as i8,
            *self.keys.get(15).unwrap() as i8,
        ];
        let result: Option<u8>;
        unsafe {
            let node_key: __m128i = _mm_loadu_si128(raw_node_key.as_ptr() as *const _);
            let key = _mm_set1_epi8(key as i8);
            let cmp = _mm_cmpeq_epi8(key, node_key);
            let mask = (1 << 16) - 1;
            let index = _mm_movemask_epi8(cmp) & mask;
            result = if index == 0 {
                None
            } else {
                Some(index as u8 - 1)
            };
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
                    Err(()) => return Err(()),
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

impl<K, V> Node16<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    fn grow(&mut self) -> Node48<K, V> {
        let mut keys = unsafe { make_array!(256, AtomicU8::new(0)) };
        let mut children = unsafe { ManuallyDrop::new(make_array!(48, ArtNode::Empty)) };
        let mut new_children_index = 0;
        for index in 0..self.header.num_children - 1 {
            let new_index = self.keys.get(index as usize).unwrap();
            let mut _key = keys.get_mut(*new_index as usize).unwrap();
            _key.store(new_children_index, Ordering::Relaxed);

            let mut _c = children.get_mut(new_children_index as usize).unwrap();
            _c = self.children.get_mut(index as usize).unwrap();

            new_children_index += 1
        }
        return Node48 {
            header: self.header.clone(),
            keys: keys,
            children: children,
            marker: Default::default(),
        };
    }

    fn shrink(&mut self) -> Node4<K, V> {
        let mut keys: [AtomicU8; 256] = unsafe { mem::uninitialized() };
        let mut children: mem::ManuallyDrop<[ArtNode<K, V>; 48]> = unsafe { mem::uninitialized() };
        let mut new_children_index = 0;
        for index in 0..15 {
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
        return Node4 {
            header: self.header.clone(),
            keys: keys,
            children: children,
            marker: Default::default(),
        };
    }
}

impl<K, V> Drop for Node16<K, V>
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
