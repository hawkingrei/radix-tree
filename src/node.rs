enum NodeType {
    NODE4,
    NODE16,
    NODE48,
    NODE256,
}

pub enum ArtNode<K, V>
where
    V: 'static + Send + Sync,
{
    Empty,

    Inner4(Box<Node4<K, V>>),
    Inner16(Box<Node16<K, V>>),
    Inner48(Box<Node48<K, V>>),
    Inner256(Box<Node256<K, V>>),
    //LeafLarge(Box<(K, V)>),
    //LeafLargeKey(Box<K>, SmallStruct<V>),
    //LeafLargeValue(SmallStruct<K>, Box<V>),
    //LeafSmall(SmallStruct<K>, SmallStruct<V>),
}

pub trait ArtKey {
    fn bytes(&self) -> &[u8];
}

pub struct NodeHeader {
    //NodeType: NodeType,
    version: Arc<AtomicUsize>, // unlock 0, lock 1
    num_children: u8,
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
        if is_obsolete(&self.version) || is_obsolete(&self.version) {
            return true;
        }
        let ver = self.version.load(Ordering::SeqCst);
        match self
            .version
            .compare_exchange(ver, ver + 1, Ordering::SeqCst, Ordering::Relaxed)
        {
            Ok(_) => {
                self.write_unlock();
            }
            Err(_) => return true,
        }
        return false;
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

    fn write_unlock(&self) {
        let ver = self.version.load(Ordering::SeqCst);
        self.version.store(ver + 3, Ordering::SeqCst);
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

trait ArtNodeTrait<K, V>
where
    V: 'static + Send + Sync,
{
    fn new() -> Self;

    //fn add_child(&mut self, node: ArtNode<K, V>, byte: u8);

    //fn clean_child(&mut self, byte: u8) -> bool;

    //#[inline]
    //fn is_full(&self) -> bool;

    //fn grow_and_add(self, leaf: ArtNode<K, V>, byte: u8) -> ArtNode<K, V>;

    //fn shrink(self) -> ArtNode<K, V>;

    //#[inline]
    //fn mut_header(&mut self) -> &mut NodeHeader;

    //#[inline]
    //fn header(&self) -> &NodeHeader;

    //#[inline]
    //fn find_child_mut(&mut self, byte: u8) -> &mut ArtNode<K, V>;

    //#[inline]
    //fn find_child(&self, byte: u8) -> Option<&ArtNode<K, V>>;

    //#[inline]
    //fn has_child(&self, byte: u8) -> bool;

    //#[inline]
    //fn to_art_node(self: Box<Self>) -> ArtNode<K, V>;
}
