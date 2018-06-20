use std::cmp;
use std::marker::PhantomData;
/// A trait for either `Owned` or `Shared` pointers.
pub trait Pointer<T> {
    /// Returns the machine representation of the pointer.
    fn into_usize(self) -> usize;

    /// Returns a new pointer pointing to the tagged pointer `data`.
    unsafe fn from_usize(data: usize) -> Self;
}

/// A pointer to an object protected by the epoch GC.
///
/// The pointer is valid for use only during the lifetime `'g`.
///
/// The pointer must be properly aligned. Since it is aligned, a tag can be stored into the unused
/// least significant bits of the address.
pub struct Shared<'g, T: 'g> {
    data: usize,
    _marker: PhantomData<(&'g (), *const T)>,
}

impl<'g, T> Clone for Shared<'g, T> {
    fn clone(&self) -> Self {
        Shared {
            data: self.data,
            _marker: PhantomData,
        }
    }
}

impl<'g, T> Copy for Shared<'g, T> {}

impl<'g, T> Pointer<T> for Shared<'g, T> {
    #[inline]
    fn into_usize(self) -> usize {
        self.data
    }

    #[inline]
    unsafe fn from_usize(data: usize) -> Self {
        Shared {
            data: data,
            _marker: PhantomData,
        }
    }
}

impl<'g, T> Shared<'g, T> {
    /// Returns a new null pointer.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_epoch::Shared;
    ///
    /// let p = Shared::<i32>::null();
    /// assert!(p.is_null());
    /// ```
    pub fn null() -> Shared<'g, T> {
        Shared {
            data: 0,
            _marker: PhantomData,
        }
    }
}

impl<'g, T> PartialEq<Shared<'g, T>> for Shared<'g, T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl<'g, T> Eq for Shared<'g, T> {}

impl<'g, T> PartialOrd<Shared<'g, T>> for Shared<'g, T> {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl<'g, T> Ord for Shared<'g, T> {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.data.cmp(&other.data)
    }
}
