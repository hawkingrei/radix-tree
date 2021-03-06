use internal::Digital;
use node::ArtNode::Empty;
use node::ArtNodeTrait;
use node::NodeHeader;
use node::{ArtKey, ArtNode};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Radix<K, V>
where
    K: Default + PartialEq + Digital + ArtKey,
    V: 'static + Send + Sync,
{
    head: ArtNode<K, V>,
    size: AtomicUsize,
    phantom: PhantomData<K>,
}

impl<K: ArtKey, T> Default for Radix<K, T>
where
    K: Default + PartialEq + Digital + ArtKey,
    T: 'static + Send + Sync,
{
    fn default() -> Self {
        Radix {
            head: ArtNode::Empty,
            size: AtomicUsize::new(0),
            phantom: Default::default(),
        }
    }
}

impl<K: ArtKey, T> Radix<K, T>
where
    K: Default + Copy + PartialEq + Digital + ArtKey,
    T: 'static + Send + Sync,
{
    fn new(level: usize) -> Self {
        Radix {
            head: ArtNode::Empty,
            size: AtomicUsize::new(0),
            phantom: Default::default(),
        }
    }

    fn insert_rec(
        parent: &ArtNode<K, T>,
        root: &mut ArtNode<K, T>,
        depth: usize,
        parent_version: usize,
        key: K,
        value: T,
    ) -> Result<(), ()> {
        let mut next_depth = depth;
        match root {
            ArtNode::Empty => print!("1"),
            ArtNode::Inner4(ptr) => loop {
                let version = match ptr.header.read_lock_or_restart() {
                    Err(_) => return Err(()),
                    Ok(version) => version,
                };
                match ptr.prefix_matches(key, depth) {
                    Ok(dep) => next_depth = dep,
                    Err(_) => {}
                };
                //if !matches!(parent, ArtNode::Empty) {
                //    if read_unlock_or_restart!(parent, parent_version) {
                //        return Err(());
                //    }
                //};
            },
            ArtNode::Inner16(ptr) => loop {
                let version = match ptr.header.read_lock_or_restart() {
                    Err(_) => return Err(()),
                    Ok(version) => version,
                };
                //if !matches!(parent, ArtNode::Empty) {
                //    if read_unlock_or_restart!(parent, parent_version) {
                //        return Err(());
                //    }
                //};
            },
            ArtNode::Inner48(ptr) => loop {
                let version = match ptr.header.read_lock_or_restart() {
                    Err(_) => return Err(()),
                    Ok(version) => version,
                };
                //if !matches!(parent, ArtNode::Empty) {
                //    if read_unlock_or_restart!(parent, parent_version) {
                //        return Err(());
                //    }
                //};
            },
            ArtNode::Inner256(ptr) => loop {
                let version = match ptr.header.read_lock_or_restart() {
                    Err(_) => return Err(()),
                    Ok(version) => version,
                };
                //if !matches!(parent, ArtNode::Empty) {
                //    if read_unlock_or_restart!(parent, parent_version) {
                //        return Err(());
                //    }
                //};
            },
            ArtNode::Value(ptr) => print!("1"),
        }
        return Err(());
    }

    fn insert(&mut self, key: K, value: T) {
        Self::insert_rec(&ArtNode::Empty, &mut self.head, 0, 0, key, value);
        self.size.fetch_add(1, Ordering::SeqCst);
    }
}
