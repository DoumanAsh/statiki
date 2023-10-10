//!Static vector
//!
use core::{mem, ptr, slice};

///Static array with `Vec`-like interface
pub struct Array<T, const C: usize> {
    inner: mem::MaybeUninit<[T; C]>,
    len: usize,
}

impl<T, const C: usize> Array<T, C> {
    #[inline]
    ///Creates new empty instance
    pub const fn new() -> Self {
        Self {
            inner: mem::MaybeUninit::uninit(),
            len: 0,
        }
    }

    #[inline]
    ///Returns length of vector.
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    ///Returns pointer to first element in underlying buffer.
    pub const fn as_ptr(&self) -> *const T {
        &self.inner as *const _ as *const _
    }

    #[inline(always)]
    ///Returns pointer to first element in underlying buffer.
    pub fn as_mut_ptr(&mut self) -> *mut T {
        &mut self.inner as *mut _ as *mut _
    }

    #[inline(always)]
    fn as_elem(&self, pos: usize) -> *const T {
        let ptr = self.as_ptr();
        unsafe {
            ptr.add(pos)
        }
    }

    #[inline(always)]
    fn as_mut_elem(&mut self, pos: usize) -> *mut T {
        let ptr = self.as_mut_ptr();
        unsafe {
            ptr.add(pos)
        }
    }

    #[inline]
    ///Retrieves reference to element without checking boundaries.
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        &*self.as_elem(index)
    }

    #[inline]
    ///Retrieves mutable reference to element without checking boundaries.
    pub unsafe fn get_unchecked_mut(&mut self, index: usize) -> &mut T {
        &mut *self.as_mut_elem(index)
    }

    #[inline]
    ///Returns immutable slice with current elements
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            slice::from_raw_parts(self.as_elem(0), self.len)
        }
    }

    #[inline]
    ///Returns mutable slice with current elements
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            slice::from_raw_parts_mut(self.as_mut_elem(0), self.len)
        }
    }

    fn inner_truncate(&mut self, len: usize) {
        if mem::needs_drop::<T>() {
            loop {
                unsafe {
                    ptr::drop_in_place(self.as_mut_elem(self.len - 1));
                }
                self.len -= 1;

                if self.len == len {
                    break;
                }
            }
        } else {
            self.len = len;
        }
    }

    ///Shortens vector, keeping the first `len` elements.
    ///
    ///Does nothing if `len` is greater or equal to vector length.
    pub fn truncate(&mut self, len: usize) {
        if len >= self.len {
            return;
        }
        self.inner_truncate(len);
    }

    ///Returns whether vector is empty.
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    ///Returns vector capacity.
    pub const fn capacity(&self) -> usize {
        C
    }

    ///Sets new length of vector.
    ///
    ///# Notes:
    ///
    ///Panics in debug mode only when `new_len` is greater than CAPACITY.
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity());
        self.len = new_len;
    }

    #[inline]
    ///Removes all elements from vector
    pub fn clear(&mut self) {
        self.truncate(0);
    }

    #[inline]
    ///Appends element at the end, without checking capacity
    pub unsafe fn push_unchecked(&mut self, value: T) {
        ptr::write(self.as_mut_elem(self.len), value);
        self.len += 1;
    }

    #[must_use]
    ///Appends element at the end.
    ///
    ///Returns `Some(T)` on capacity overflow
    pub fn push(&mut self, value: T) -> Option<T> {
        match self.len == self.capacity() {
            true => Some(value),
            false => unsafe {
                self.push_unchecked(value);
                None
            },
        }
    }

    #[inline]
    ///Unconditionally retrieves element from vector.
    pub unsafe fn pop_unchecked(&mut self) -> T {
        let result = ptr::read(self.as_elem(self.len - 1));

        self.len -= 1;

        result
    }

    ///Pops element out of vector.
    pub fn pop(&mut self) -> Option<T> {
        match self.len {
            0 => None,
            _ => unsafe {
                Some(self.pop_unchecked())
            }
        }
    }

    ///Removes element at `index` by swapping it with last element, and popping out.
    pub unsafe fn swap_remove_unchecked(&mut self, index: usize) -> T {
        ptr::swap(self.as_mut_elem(index), self.as_mut_elem(self.len - 1));
        self.pop_unchecked()
    }

    ///Removes element at `index` by swapping it with last element, and popping out.
    ///
    ///## Note:
    ///
    ///Panics when `index` is out of bounds
    pub fn swap_remove(&mut self, index: usize) -> T {
        assert!(index < self.len);
        unsafe {
            self.swap_remove_unchecked(index)
        }
    }

    ///Resizes vector with provided `value`
    ///
    ///If `new_len` is greater than `len`, the `Array` is extended by the difference, with each
    ///additional slot filled with value. If `new_len` is less than `len`, the `Array` is simply
    ///truncated.
    pub unsafe fn resize_unchecked(&mut self, new_len: usize, value: T) where T: Clone {
        match new_len > self.len() {
            true => while self.len() < new_len {
                self.push_unchecked(value.clone());
            },
            false => self.truncate(new_len),
        }
    }

    #[inline]
    ///Resizes vector with provided `value`
    ///
    ///If `new_len` is greater than `len`, the `Array` is extended by the difference, with each
    ///additional slot filled with value. If `new_len` is less than `len`, the `Array` is simply
    ///truncated.
    ///
    ///## Note:
    ///
    ///Panics if `new_len` is greater than `CAPACITY`
    pub fn resize(&mut self, new_len: usize, value: T) where T: Clone {
        assert!(new_len <= self.capacity());
        unsafe {
            self.resize_unchecked(new_len, value);
        }
    }

    ///Resizes vector with default values.
    ///
    ///If `new_len` is greater than `len`, the `Array` is extended by the difference, with each
    ///additional slot filled with value. If `new_len` is less than `len`, the `Array` is simply
    ///truncated.
    pub unsafe fn resize_default_unchecked(&mut self, new_len: usize) where T: Default {
        match new_len > self.len() {
            true => while self.len() < new_len {
                self.push_unchecked(T::default());
            },
            false => self.truncate(new_len),
        }
    }

    #[inline]
    ///Resizes vector with default values.
    ///
    ///If `new_len` is greater than `len`, the `Array` is extended by the difference, with each
    ///additional slot filled with value. If `new_len` is less than `len`, the `Array` is simply
    ///truncated.
    ///
    ///## Note:
    ///
    ///Panics if `new_len` is greater than `CAPACITY`
    pub fn resize_default(&mut self, new_len: usize) where T: Default {
        assert!(new_len <= self.capacity());
        unsafe {
            self.resize_default_unchecked(new_len);
        }
    }
}

impl<T, const C: usize> Drop for Array<T, C> {
    #[inline]
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const C: usize> core::ops::Deref for Array<T, C> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const C: usize> core::ops::DerefMut for Array<T, C> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

impl<T, const C: usize> AsRef<Array<T, C>> for Array<T, C> {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T, const C: usize> AsMut<Array<T, C>> for Array<T, C> {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, const C: usize> AsRef<[T]> for Array<T, C> {
    #[inline]
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<T, const C: usize> AsMut<[T]> for Array<T, C> {
    #[inline]
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

impl<T: core::fmt::Debug, const C: usize> core::fmt::Debug for Array<T, C> {
    #[inline(always)]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

impl<T: Clone, const C: usize> Clone for Array<T, C> {
    fn clone(&self) -> Self {
        let mut result = Self {
            inner: mem::MaybeUninit::uninit(),
            len: self.len,

        };

        unsafe {
            self.inner.as_ptr().copy_to_nonoverlapping(result.inner.as_mut_ptr(), 1);
        }
        result
    }
}

impl<T: PartialEq, const C: usize> PartialEq for Array<T, C> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: PartialEq, const C: usize> PartialEq<[T]> for Array<T, C> {
    #[inline]
    fn eq(&self, other: &[T]) -> bool {
        self.as_slice() == other
    }
}

impl<T: PartialEq, const C: usize> PartialEq<&'_ [T]> for Array<T, C> {
    #[inline]
    fn eq(&self, other: &&[T]) -> bool {
        self.as_slice() == *other
    }
}

impl<T: Eq, const C: usize> Eq for Array<T, C> {
}

#[cfg(feature = "std")]
impl<const C: usize> std::io::Write for Array<u8, C> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let write_len = core::cmp::min(self.capacity() - self.len(), buf.len());
        let dest = self.as_mut_elem(self.len);
        let src = buf.as_ptr();
        unsafe {
            ptr::copy_nonoverlapping(src, dest, write_len);
        }
        self.len += write_len;

        Ok(write_len)
    }

    #[inline]
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct ArrayConsumer<T, const C: usize> {
    inner: Array<T, C>,
    cursor: usize,
}

impl<T, const C: usize> Iterator for ArrayConsumer<T, C> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        if self.cursor < self.inner.len() {
            let result = unsafe {
                ptr::read(self.inner.as_elem(self.cursor))
            };
            self.cursor += 1;
            Some(result)
        } else {
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.inner.len() - self.cursor;
        (size, Some(size))
    }
}

impl<T, const C: usize> Drop for ArrayConsumer<T, C> {
    fn drop(&mut self) {
        if mem::needs_drop::<T>() {
            while let Some(_) = self.next() {
            }
        }
        unsafe {
            self.inner.set_len(0);
        }
    }
}

impl<T, const C: usize> IntoIterator for Array<T, C> {
    type Item = T;
    type IntoIter = ArrayConsumer<T, C>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        ArrayConsumer {
            inner: self,
            cursor: 0,
        }
    }
}
