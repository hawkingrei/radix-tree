use epoch::guard::Guard;
use epoch::pointer::Pointer;
use epoch::shared::Shared;
use std::boxed::Box;
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ops::Deref;

/// Returns a bitmask containing the unused least significant bits of an aligned pointer to `T`.
#[inline]
fn low_bits<T>() -> usize {
    (1 << mem::align_of::<T>().trailing_zeros()) - 1
}

/// Panics if the pointer is not properly unaligned.
#[inline]
fn ensure_aligned<T>(raw: *const T) {
    assert_eq!(raw as usize & low_bits::<T>(), 0, "unaligned pointer");
}

/// An owned heap-allocated object.
///
/// This type is very similar to `Box<T>`.
///
/// The pointer must be properly aligned. Since it is aligned, a tag can be stored into the unused
/// least significant bits of the address.
pub struct Owned<T> {
    data: usize,
    _marker: PhantomData<Box<T>>,
}

impl<T> Pointer<T> for Owned<T> {
    #[inline]
    fn into_usize(self) -> usize {
        let data = self.data;
        mem::forget(self);
        data
    }

    /// Returns a new pointer pointing to the tagged pointer `data`.
    ///
    /// # Panics
    ///
    /// Panics if the data is zero in debug mode.
    #[inline]
    unsafe fn from_usize(data: usize) -> Self {
        debug_assert!(data != 0, "converting zero into `Owned`");
        Owned {
            data: data,
            _marker: PhantomData,
        }
    }
}

impl<T> From<T> for Owned<T> {
    fn from(t: T) -> Self {
        Owned::new(t)
    }
}

impl<T> Owned<T> {
    /// Allocates `value` on the heap and returns a new owned pointer pointing to it.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_epoch::Owned;
    ///
    /// let o = Owned::new(1234);
    /// ```
    pub fn new(value: T) -> Owned<T> {
        Self::from(Box::new(value))
    }

    /// Returns a new owned pointer pointing to `raw`.
    ///
    /// This function is unsafe because improper use may lead to memory problems. Argument `raw`
    /// must be a valid pointer. Also, a double-free may occur if the function is called twice on
    /// the same raw pointer.
    ///
    /// # Panics
    ///
    /// Panics if `raw` is not properly aligned.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_epoch::Owned;
    ///
    /// let o = unsafe { Owned::from_raw(Box::into_raw(Box::new(1234))) };
    /// ```
    pub unsafe fn from_raw(raw: *mut T) -> Owned<T> {
        ensure_aligned(raw);
        Self::from_usize(raw as usize)
    }

    /// Converts the owned pointer into a [`Shared`].
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_epoch::{self as epoch, Owned};
    ///
    /// let o = Owned::new(1234);
    /// let guard = &epoch::pin();
    /// let p = o.into_shared(guard);
    /// ```
    ///
    /// [`Shared`]: struct.Shared.html
    pub fn into_shared<'g>(self, _: &'g Guard) -> Shared<'g, T> {
        unsafe { Shared::from_usize(self.into_usize()) }
    }

    /// Converts the owned pointer into a `Box`.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_epoch::{self as epoch, Owned};
    ///
    /// let o = Owned::new(1234);
    /// let b: Box<i32> = o.into_box();
    /// assert_eq!(*b, 1234);
    /// ```
    pub fn into_box(self) -> Box<T> {
        let raw = self.data as *mut T;
        mem::forget(self);
        unsafe { Box::from_raw(raw) }
    }
}

impl<T> From<Box<T>> for Owned<T> {
    /// Returns a new owned pointer pointing to `b`.
    ///
    /// # Panics
    ///
    /// Panics if the pointer (the `Box`) is not properly aligned.
    ///
    /// # Examples
    ///
    /// ```
    /// use crossbeam_epoch::Owned;
    ///
    /// let o = unsafe { Owned::from_raw(Box::into_raw(Box::new(1234))) };
    /// ```
    fn from(b: Box<T>) -> Self {
        unsafe { Self::from_raw(Box::into_raw(b)) }
    }
}

impl<T> Drop for Owned<T> {
    fn drop(&mut self) {
        let raw = self.data as *mut T;
        unsafe {
            drop(Box::from_raw(raw));
        }
    }
}

impl<T> fmt::Debug for Owned<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let raw = self.data as *mut T;
        f.debug_struct("Owned").field("raw", &raw).finish()
    }
}
