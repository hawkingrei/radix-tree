use internal::Digital;
use node;
use node::{ArtNode, ArtNodeTrait, NodeHeader};
use std::cmp::PartialEq;
use std::marker::PhantomData;
use std::mem;

pub struct Node4<'a, K, T>
where
    K: Default + PartialEq + Digital<'a>,
    T: 'static + Send + Sync,
{
    header: NodeHeader,
    keys: Vec<K>,
    children: Vec<ArtNode<'a, K, T>>,
    marker: PhantomData<&'a T>,
}

impl<'a, K, V> ArtNodeTrait<'a, K, V> for Node4<'a, K, V>
where
    K: Default + PartialEq + Digital<'a>,
    V: 'static + Send + Sync,
{
    fn new() -> Self {
        Node4 {
            header: NodeHeader::new(),
            keys: rep_no_copy!(K; Default::default(); 4),
            children: rep_no_copy!(ArtNode<K, V>; ArtNode::Empty; 4),
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
    ) -> &mut Result<&mut ArtNode<K, V>, bool> {
        let mut version = 0;
        loop {
            match self.header.read_lock_or_restart() {
                Ok(ver) => version = ver,
                Err(true) => continue,
                Err(false) => return &mut Err((false)),
            }
        }
        let key = byte.to_le().to_bytes()[level];
        let mut index = 0;
        let mut result: Option<u8>;
        for rkey in self.keys.iter() {
            if index + 1 <= self.header.num_children {
                result = None;
                break;
            }
            unsafe {
                if *rkey == key {
                    result = Some(key);
                    break;
                }
                index += 1;
            };
        }
        match result {
            Some(index) => {
                let next_node = self.children.get(index as usize);
                loop {
                    match self.header.read_lock_or_restart() {
                        Ok(ver) => if version == ver {
                            match next_node {
                                None => return &mut Err(true),
                            }
                        } else {
                            return &mut Err(true);
                        },
                        Err(true) => continue,
                        Err(false) => return &mut Err(false),
                    }
                }
            }
            None => return &mut Err(false),
        }
    }
}

impl<'a, K, V> Drop for Node4<'a, K, V>
where
    K: Default + PartialEq + Digital<'a>,
    V: 'static + Send + Sync,
{
    fn drop(&mut self) {
        for i in 0..self.header.num_children {
            drop(&mut self.children[i as usize]);
        }
    }
}
