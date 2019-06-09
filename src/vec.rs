//!Static vector
//!
//!Implementation of `Vec` backed by static memory.
//!Its resize capabilities is limited by specified `CAPACITY`
//!
//!See [API](struct.Vec.html) of pre-generated `Vec`.
//!## Usage:
//!
//!```rust
//!statiki::declare_vec!(512); //Creates Vec with CAPACITY 512
//!
//!let mut queue = Vec::new();
//!assert_eq!(queue.capacity(), 512);
//!assert!(queue.is_empty());
//!
//!queue.push(1);
//!while !queue.is_empty() {
//!    println!("Elem={}", queue.pop().expect("Element"));
//!}
//!```


#[macro_export]
///Generates `Vec` with specified capacity
macro_rules! declare_vec {
    ($capacity:expr) => {
        use core::{mem, ptr, slice};

        ///Automatically generated Vec
        pub struct Vec<T> {
            inner: mem::MaybeUninit<[T; $capacity]>,
            len: usize,
        }

        #[allow(unused)]
        impl<T> Vec<T> {
            ///Capacity.
            pub const CAPACITY: usize = $capacity;

            ///Creates new instance.
            pub const fn new() -> Self {
                Vec {
                    inner: mem::MaybeUninit::uninit(),
                    len: 0,
                }
            }

            #[inline(always)]
            ///Returns vector's current length.
            pub fn len(&self) -> usize {
                self.len
            }

            ///Returns pointer to underlying buffer.
            pub fn as_ptr(&self) -> *const T {
                unsafe {
                    self.inner.as_ptr() as *const T
                }
            }

            ///Returns mutable pointer to underlying buffer.
            pub fn as_mut_ptr(&mut self) -> *mut T {
                unsafe {
                    self.inner.as_mut_ptr() as *mut T
                }
            }


            fn as_elem(&self, pos: usize) -> *const T {
                let ptr = self.as_ptr();
                unsafe {
                    ptr.offset(pos as isize)
                }
            }

            fn as_mut_elem(&mut self, pos: usize) -> *mut T {
                let ptr = self.as_mut_ptr();
                unsafe {
                    ptr.offset(pos as isize)
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

            ///Shortens vector, keeping the first `len` elements.
            ///
            ///Does nothing if `len` is greater or equal to vector length.
            pub fn truncate(&mut self, len: usize) {
                if len >= self.len {
                    return;
                }

                let cursor = self.as_mut_elem(self.len);

                for _ in len..self.len {
                    self.len -= 1;
                    unsafe {
                        cursor.offset(-1);
                        ptr::drop_in_place(cursor);
                    }
                }
            }

            ///Returns whether vector is empty.
            pub fn is_empty(&self) -> bool {
                self.len == 0
            }

            ///Returns vector capacity.
            pub const fn capacity(&self) -> usize {
                Self::CAPACITY
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

            ///Appends element at the end.
            ///
            ///Returns `Some(T)` on capacity overflow
            pub fn push(&mut self, value: T) -> Option<T> {
                match self.len == self.capacity() {
                    true => Some(value),
                    false => {
                        unsafe {
                            self.push_unchecked(value);
                        }
                        None
                    },
                }
            }

            #[inline]
            ///Appends element at the end, without checking capacity
            pub unsafe fn push_unchecked(&mut self, value: T) {
                ptr::write(self.as_mut_elem(self.len), value);
                self.len += 1;
            }

            ///Pops element out of vector.
            pub fn pop(&mut self) -> Option<T> {
                if self.len == 0 {
                    None
                } else {
                    unsafe {
                        Some(self.pop_unchecked())
                    }
                }
            }

            ///Unconditionally retrieves element from vector.
            pub unsafe fn pop_unchecked(&mut self) -> T {
                let mut result = mem::MaybeUninit::uninit();

                ptr::copy_nonoverlapping(self.as_elem(self.len - 1), result.as_mut_ptr(), 1);

                self.len -= 1;

                result.assume_init()
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

            ///Removes element at `index` by swapping it with last element, and popping out.
            pub unsafe fn swap_remove_unchecked(&mut self, index: usize) -> T {
                ptr::swap(self.as_mut_elem(index), self.as_mut_elem(self.len - 1));
                self.pop_unchecked()
            }

            #[inline]
            ///Resizes vector with provided `value`
            ///
            ///If `new_len` is greater than `len`, the `Vec` is extended by the difference, with each
            ///additional slot filled with value. If `new_len` is less than `len`, the `Vec` is simply
            ///truncated.
            ///
            ///## Note:
            ///
            ///Panics if `new_len` is greater than `CAPACITY`
            pub fn resize(&mut self, new_len: usize, value: T) where T: Clone {
                assert!(new_len <= Self::CAPACITY);
                unsafe {
                    self.resize_unchecked(new_len, value);
                }
            }

            ///Resizes vector with provided `value`
            ///
            ///If `new_len` is greater than `len`, the `Vec` is extended by the difference, with each
            ///additional slot filled with value. If `new_len` is less than `len`, the `Vec` is simply
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
            ///Resizes vector with default values.
            ///
            ///If `new_len` is greater than `len`, the `Vec` is extended by the difference, with each
            ///additional slot filled with value. If `new_len` is less than `len`, the `Vec` is simply
            ///truncated.
            ///
            ///## Note:
            ///
            ///Panics if `new_len` is greater than `CAPACITY`
            pub fn resize_default(&mut self, new_len: usize) where T: Default {
                assert!(new_len <= Self::CAPACITY);
                unsafe {
                    self.resize_default_unchecked(new_len);
                }
            }


            ///Resizes vector with default values.
            ///
            ///If `new_len` is greater than `len`, the `Vec` is extended by the difference, with each
            ///additional slot filled with value. If `new_len` is less than `len`, the `Vec` is simply
            ///truncated.
            pub unsafe fn resize_default_unchecked(&mut self, new_len: usize) where T: Default {
                match new_len > self.len() {
                    true => while self.len() < new_len {
                        self.push_unchecked(T::default());
                    },
                    false => self.truncate(new_len),
                }
            }
        }

        impl<T> Drop for Vec<T> {
            fn drop(&mut self) {
                self.clear();
            }
        }

        impl<T> core::ops::Deref for Vec<T> {
            type Target = [T];

            fn deref(&self) -> &Self::Target {
                self.as_slice()
            }
        }

        impl<T> core::ops::DerefMut for Vec<T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.as_mut_slice()
            }
        }

        impl<T> AsRef<Vec<T>> for Vec<T> {
            #[inline]
            fn as_ref(&self) -> &Self {
                self
            }
        }

        impl<T> AsMut<Vec<T>> for Vec<T> {
            #[inline]
            fn as_mut(&mut self) -> &mut Self {
                self
            }
        }

        impl<T> AsRef<[T]> for Vec<T> {
            #[inline]
            fn as_ref(&self) -> &[T] {
                self
            }
        }

        impl<T> AsMut<[T]> for Vec<T> {
            #[inline]
            fn as_mut(&mut self) -> &mut [T] {
                self
            }
        }

        impl<T: core::fmt::Debug> core::fmt::Debug for Vec<T> {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                match self.len {
                    0 => write!(f, "[]"),
                    len => {
                        write!(f, "[")?;
                        for idx in 0..len-1 {
                            let elem = unsafe {
                                self.get_unchecked(idx)
                            };

                            write!(f, "{:?}, ", elem)?;
                        }
                        write!(f, "{:?}]", unsafe { self.get_unchecked(self.len-1) })
                    },
                }
            }
        }
    }
}

#[cfg(feature = "docs")]
declare_vec!(512);

#[cfg(test)]
mod tests {
    use core::fmt::Write;
    declare_vec!(512);

    #[test]
    fn test_queue() {
        let mut vec = Vec::new();
        assert_eq!(vec.capacity(), 512);
        assert!(vec.is_empty());
        assert_eq!(format!("{:?}", &vec), "[]");

        vec.push(15);
        assert!(!vec.is_empty());

        let elem = vec.pop().expect("To get value");
        assert_eq!(15, elem);
        assert!(vec.pop().is_none());

        for idx in 0..vec.capacity() {
            assert!(vec.push(idx).is_none());
        }

        assert!(vec.push(500).is_some());

        for idx in (0..vec.capacity()).rev() {
            let elem = vec.pop().expect("To get value");
            assert_eq!(idx, elem);
        }

        assert!(vec.pop().is_none());
        vec.clear();

        let mut expected_format = String::new();
        let _ = write!(&mut expected_format, "[");
        for idx in 0..vec.capacity() {
            assert!(vec.push(idx).is_none());
            let _ = write!(&mut expected_format, "{}, ", idx);
        }
        expected_format.pop();
        expected_format.pop();
        let _ = write!(&mut expected_format, "]");

        assert_eq!(format!("{:?}", &vec), expected_format);

        vec.resize(vec.capacity() / 2, 1);
        assert_eq!(vec.len(), vec.capacity() / 2);

        vec.clear();
        assert!(vec.is_empty());
        assert!(vec.pop().is_none());

        vec.resize(vec.capacity(), 1);
        assert_eq!(vec.len(), vec.capacity());

        for _ in 0..vec.capacity() {
            let item = vec.pop().expect("To get value");
            assert_eq!(item, 1);
        }

        assert!(vec.is_empty());
        assert!(vec.pop().is_none());

        vec.resize_default(vec.capacity() / 2);
        assert_eq!(vec.len(), vec.capacity() / 2);

        for _ in 0..vec.capacity() / 2 {
            let item = vec.pop().expect("To get value");
            assert_eq!(item, 0);
        }

        assert!(vec.is_empty());
        assert!(vec.pop().is_none());
    }
}
