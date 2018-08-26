use internal::Digital;
use node16::Node16;
use node256::Node256;
use node4::Node4;
use node48::Node48;
use std::arch::x86_64::_mm_pause;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::{mem, ptr};

const MAX_PREFIX_LEN: usize = 6;

enum NodeType {
    NODE4,
    NODE16,
    NODE48,
    NODE256,
}

pub enum ArtNode<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    Empty,

    Inner4(Box<Node4<K, V>>),
    Inner16(Box<Node16<K, V>>),
    Inner48(Box<Node48<K, V>>),
    Inner256(Box<Node256<K, V>>),
    Value(usize),
    //LeafLarge(Box<(K, V)>),
    //LeafLargeKey(Box<K>, SmallStruct<V>),
    //LeafLargeValue(SmallStruct<K>, Box<V>),
    //LeafSmall(SmallStruct<K>, SmallStruct<V>)
}

pub trait ArtKey {
    fn bytes(&self) -> &[u8];
}

#[derive(Clone)]
pub struct NodeHeader {
    //NodeType: NodeType,
    version: Arc<AtomicUsize>, // unlock 0, lock 1
    pub num_children: u8,
    partial: [u8; MAX_PREFIX_LEN],
    partial_len: usize,
}

pub fn is_locked(version: &Arc<AtomicUsize>) -> bool {
    version.load(Ordering::SeqCst) & 0b10 == 0b10
}

pub fn is_obsolete(version: &Arc<AtomicUsize>) -> bool {
    version.load(Ordering::SeqCst) & 1 == 1
}

impl NodeHeader {
    pub fn new() -> Self {
        NodeHeader {
            version: Arc::new(AtomicUsize::new(0)),
            num_children: 0,
            partial_len: 0,
            partial: unsafe { mem::uninitialized() },
        }
    }

    pub fn get_partial_len(&self) -> usize {
        self.partial_len
    }

    #[inline]
    pub fn read_version(&self) -> usize {
        return self.version.load(Ordering::SeqCst);
    }

    pub fn write_lock_or_restart(&self) -> bool {
        loop {
            let mut ver = self.version.load(Ordering::SeqCst);
            while is_locked(&self.version) {
                unsafe {
                    _mm_pause();
                    ver = self.version.load(Ordering::SeqCst);
                }
            }

            if is_obsolete(&self.version) {
                return true;
            }

            match self.version.compare_exchange_weak(
                ver,
                ver + 0b10,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(_) => continue,
            }
        }
        false
    }

    pub fn lock_version_or_restart(&self) -> bool {
        if is_locked(&self.version) || is_obsolete(&self.version) {
            return true;
        }
        let ver = self.version.load(Ordering::SeqCst);
        match self.version.compare_exchange(
            ver,
            ver.wrapping_add(1),
            Ordering::SeqCst,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                self.write_unlock();
            }
            Err(_) => return true,
        }
        return false;
    }

    pub fn read_unlock_or_restart(header: usize, version: usize) -> bool {
        if header == version {
            return false;
        }
        return true;
    }

    pub fn prefix_match<K: Digital>(&self, key: K, depth: usize) -> bool {
        let k = key.to_le_bytes();
        for i in 0..self.partial_len {
            if *k.get(i + depth).unwrap() != self.partial[i] {
                return false;
            }
        }
        return true;
    }

    pub fn compute_prefix_match<K: ArtKey>(&self, key: &K, depth: usize) -> usize {
        for i in 0..self.partial_len {
            if key.bytes()[i + depth] != self.partial[i] {
                return i;
            }
        }
        self.partial_len
    }

    pub fn is_locked(&self) -> bool {
        is_locked(&self.version)
    }

    pub fn read_lock_or_restart(&self) -> Result<usize, ()> {
        if is_locked(&self.version) {
            return Err(());
        }
        if is_obsolete(&self.version) {
            return Err(());
        }
        let ver = self.version.load(Ordering::SeqCst);
        return Ok(ver);
    }

    fn write_unlock(&self) {
        let ver = self.version.load(Ordering::SeqCst);
        self.version.store(ver.wrapping_add(2), Ordering::SeqCst);
    }

    pub fn write_unlock_obsolete(&self) {
        let ver = self.version.load(Ordering::SeqCst);
        self.version.store(ver.wrapping_add(3), Ordering::SeqCst);
    }

    pub fn upgrade_to_write_lock_or_restart(&mut self, version: usize) -> Result<(), ()> {
        if self
            .version
            .compare_and_swap(version, version + 2, Ordering::SeqCst)
            != version + 2
        {
            return Err(());
        }
        return Ok(());
    }

    pub fn upgrade_to_write_lock_or_write_unlock_and_restart(
        &mut self,
        version: usize,
        header: &mut NodeHeader,
    ) -> Result<(), ()> {
        if self
            .version
            .compare_and_swap(version, version + 2, Ordering::SeqCst)
            == version + 2
        {
            header.write_unlock();
            return Err(());
        }
        return Ok(());
    }
}

#[test]
fn lock() {
    let mut header = NodeHeader::new();
    assert_eq!(header.is_locked(), false);
    assert_eq!(header.write_lock_or_restart(), false);
    assert_eq!(header.is_locked(), true);
    header.write_unlock();
    assert_eq!(header.is_locked(), false);
}

pub trait ArtNodeTrait<K, V>
where
    K: Default + PartialEq + Digital,
    V: 'static + Send + Sync,
{
    fn new() -> Self;

    fn add_child(&mut self, node: ArtNode<K, V>, byte: u8);

    //fn clean_child(&mut self, byte: u8) -> bool;

    fn prefix_matches(&self, key: K, level: usize) -> Result<usize, usize>;

    fn get_version(&self) -> usize;

    #[inline]
    fn is_full(&self) -> bool;

    fn change(&mut self, key: u8, val: ArtNode<K, V>) -> bool;

    fn has_child(&self, byte: u8) -> bool;

    //fn grow_and_add(self, leaf: ArtNode<K, V>, byte: u8) -> ArtNode<K, V>;

    //fn shrink(self) -> ArtNode<K, V>;

    //#[inline]
    //fn mut_header(&mut self) -> &mut NodeHeader;

    //#[inline]
    //fn header(&self) -> &NodeHeader;

    #[inline]
    fn find_child_mut(
        &mut self,
        byte: usize,
        level: usize,
        parent: ArtNode<K, V>,
        version_parent: usize,
    ) -> Result<&mut ArtNode<K, V>, ()>;

    fn insert_and_unlock(&self, parent_node: Self, key: u8) -> (ArtNode<K, V>, bool);

    //#[inline]
    //fn find_child(&self, byte: u8) -> Option<&ArtNode<K, V>>;

    //#[inline]
    //fn has_child(&self, byte: u8) -> bool;

    //#[inline]
    //fn to_art_node(self: Box<Self>) -> ArtNode<K, V>;
}
