//! The global data and participant for garbage collection.
//!
//! # Registration
//!
//! In order to track all participants in one place, we need some form of participant
//! registration. When a participant is created, it is registered to a global lock-free
//! singly-linked list of registries; and when a participant is leaving, it is unregistered from the
//! list.
//!
//! # Pinning
//!
//! Every participant contains an integer that tells whether the participant is pinned and if so,
//! what was the global epoch at the time it was pinned. Participants also hold a pin counter that
//! aids in periodic global epoch advancement.
//!
//! When a participant is pinned, a `Guard` is returned as a witness that the participant is pinned.
//! Guards are necessary for performing atomic operations, and for freeing/dropping locations.
//!
//! # Thread-local bag
//!
//! Objects that get unlinked from concurrent data structures must be stashed away until the global
//! epoch sufficiently advances so that they become safe for destruction. Pointers to such objects
//! are pushed into a thread-local bag, and when it becomes full, the bag is marked with the current
//! global epoch and pushed into the global queue of bags. We store objects in thread-local storages
//! for amortizing the synchronization cost of pushing the garbages to a global queue.
//!
//! # Global queue
//!
//! Whenever a bag is pushed into a queue, the objects in some bags in the queue are collected and
//! destroyed along the way. This design reduces contention on data structures. The global queue
//! cannot be explicitly accessed: the only way to interact with it is by calling functions
//! `defer()` that adds an object tothe thread-local bag, or `collect()` that manually triggers
//! garbage collection.
//!
//! Ideally each instance of concurrent data structure may have its own queue that gets fully
//! destroyed as soon as the data structure gets dropped.
use arrayvec::ArrayVec;
use epoch::deferred::Deferred;
use epoch::epoch::Epoch;

/// Maximum number of objects a bag can contain.
#[cfg(not(feature = "sanitize"))]
const MAX_OBJECTS: usize = 64;
#[cfg(feature = "sanitize")]
const MAX_OBJECTS: usize = 4;

/// A bag of deferred functions.
#[derive(Default, Debug)]
pub struct Bag {
    /// Stashed objects.
    deferreds: ArrayVec<[Deferred; MAX_OBJECTS]>,
}

/// `Bag::try_push()` requires that it is safe for another thread to execute the given functions.
unsafe impl Send for Bag {}

impl Bag {
    /// Returns a new, empty bag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the bag is empty.
    pub fn is_empty(&self) -> bool {
        self.deferreds.is_empty()
    }

    /// Attempts to insert a deferred function into the bag.
    ///
    /// Returns `Ok(())` if successful, and `Err(deferred)` for the given `deferred` if the bag is
    /// full.
    ///
    /// # Safety
    ///
    /// It should be safe for another thread to execute the given function.
    pub unsafe fn try_push(&mut self, deferred: Deferred) -> Result<(), Deferred> {
        self.deferreds.try_push(deferred).map_err(|e| e.element())
    }

    /// Seals the bag with the given epoch.
    fn seal(self, epoch: Epoch) -> SealedBag {
        SealedBag { epoch, bag: self }
    }
}

impl Drop for Bag {
    fn drop(&mut self) {
        // Call all deferred functions.
        for deferred in self.deferreds.drain(..) {
            deferred.call();
        }
    }
}

/// A pair of an epoch and a bag.
#[derive(Default, Debug)]
struct SealedBag {
    epoch: Epoch,
    bag: Bag,
}

/// It is safe to share `SealedBag` because `is_expired` only inspects the epoch.
unsafe impl Sync for SealedBag {}

impl SealedBag {
    /// Checks if it is safe to drop the bag w.r.t. the given global epoch.
    fn is_expired(&self, global_epoch: Epoch) -> bool {
        // A pinned participant can witness at most one epoch advancement. Therefore, any bag that
        // is within one epoch of the current one cannot be destroyed yet.
        global_epoch.wrapping_sub(self.epoch) >= 2
    }
}
