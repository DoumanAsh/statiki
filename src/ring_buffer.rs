//!Static Circular Ring Buffer
//!
//!Once it reaches the end of its capacity, it starts over-writing elements from the beggining
//!For simplicity sake it uses macro to generate appropriate code instead of generic integer hacks.
//!See [API](struct.RingBuffer.html) of pre-generated `RingBuffer`.
//!
//!## Usage:
//!
//!```rust
//!statiki::declare_ring_buffer!(512); //Creates RingBuffer with CAPACITY 512
//!
//!let mut queue = RingBuffer::new();
//!assert_eq!(queue.capacity(), 512);
//!assert!(queue.is_empty());
//!
//!queue.push(1);
//!while !queue.is_empty() {
//!    println!("Elem={}", queue.pop().expect("Element"));
//!}
//!```

#[macro_export]
///Generates `RingBuffer` with specified capacity.
///
///Capacity must be power of two.
///For now due to lack of static assertion, it panics in debug mode when attempting to push/pop value
macro_rules! declare_ring_buffer {
    ($capacity:expr) => {
        use core::{mem, ptr};

        ///Automatically generated Ring buffer
        pub struct RingBuffer<T> {
            inner: mem::MaybeUninit<[T; $capacity]>,
            read: usize,
            write: usize,
        }

        impl<T> RingBuffer<T> {
            ///Capacity.
            pub const CAPACITY: usize = $capacity;

            ///Creates new instnace
            pub const fn new() -> Self {
                RingBuffer {
                    inner: mem::MaybeUninit::uninit(),
                    read: 0,
                    write: 0,
                }
            }

            fn as_elem(&self, pos: usize) -> *const T {
                unsafe {
                    let ptr = self.inner.as_ptr() as *const T;
                    ptr.offset(self.mask_idx(pos) as isize)
                }
            }

            fn as_mut_elem(&mut self, pos: usize) -> *mut T {
                unsafe {
                    let ptr = self.inner.as_mut_ptr() as *mut T;
                    ptr.offset(self.mask_idx(pos) as isize)
                }
            }

            ///Returns the number of elements in buffer.
            pub const fn size(&self) -> usize {
                self.write - self.read
            }

            ///Retrieves buffer capacity.
            pub const fn capacity(&self) -> usize {
                Self::CAPACITY
            }

            const fn mask_idx(&self, idx: usize) -> usize {
                idx & (Self::CAPACITY - 1)
            }

            ///Returns whether buffer is empty.
            pub const fn is_empty(&self) -> bool {
                self.write == self.read
            }

            ///Returns whether buffer is empty.
            pub const fn is_full(&self) -> bool {
                self.size() == self.capacity()
            }

            ///Removes all elements from the buffer.
            pub fn clear(&mut self) {
                if mem::needs_drop::<T>() {
                    for _ in 0..self.size() {
                        unsafe {
                            self.pop_unchecked();
                        }
                    }
                } else {
                    self.read = self.write
                }
            }

            ///Adds new element.
            pub fn push(&mut self, value: T) {
                debug_assert!((Self::CAPACITY & (Self::CAPACITY - 1)) == 0, "Capacity is not power of 2");

                unsafe {
                    if self.is_full() {
                        self.pop_unchecked();
                    }
                    ptr::write(self.as_mut_elem(self.write), value);
                }
                self.write = self.write.wrapping_add(1);
            }

            ///Pops element out of buffer.
            pub fn pop(&mut self) -> Option<T> {
                debug_assert!((Self::CAPACITY & (Self::CAPACITY - 1)) == 0, "Capacity is not power of 2");

                if self.is_empty() {
                    None
                } else {
                    unsafe {
                        Some(self.pop_unchecked())
                    }
                }
            }

            ///Unconditionally retrieves element from buffer.
            pub unsafe fn pop_unchecked(&mut self) -> T {
                let mut result = mem::MaybeUninit::uninit();

                ptr::copy_nonoverlapping(self.as_elem(self.read), result.as_mut_ptr(), 1);

                self.read = self.read.wrapping_add(1);

                result.assume_init()
            }
        }

        impl<T> Drop for RingBuffer<T> {
            fn drop(&mut self) {
                self.clear();
            }
        }

    }
}

#[cfg(feature = "docs")]
declare_ring_buffer!(512);

#[cfg(test)]
mod tests {
    declare_ring_buffer!(512);

    #[test]
    fn test_ring_buffer() {
        let mut queue = RingBuffer::<usize>::new();
        assert_eq!(queue.capacity(), 512);
        assert_eq!(queue.size(), 0);
        assert!(queue.is_empty());

        for idx in 0..queue.capacity()+9 {
            queue.push(idx);
        }
        assert!(!queue.is_empty());
        assert_eq!(queue.size(), 512);

        for expected_item in 9..queue.capacity()+9 {
            let value = queue.pop().expect("Value");
            assert_eq!(value, expected_item);
        }

        assert!(queue.pop().is_none());
        assert!(queue.is_empty());
        assert_eq!(queue.size(), 0);

        for idx in 0..queue.capacity()+9 {
            queue.push(idx);
        }
        assert!(!queue.is_empty());

        queue.clear();
        assert!(queue.pop().is_none());
        assert!(queue.is_empty());
        assert_eq!(queue.size(), 0);

        for idx in 0..9 {
            queue.push(idx);
        }
        assert!(!queue.is_empty());
        assert_eq!(queue.size(), 9);
        for expected_item in 0..9 {
            let value = queue.pop().expect("Value");
            assert_eq!(value, expected_item);
        }
    }
}
