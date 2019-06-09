//!Static Ring Buffer

#[macro_export]
///Generates `RingBuffer` with specified capacity
macro_rules! declare_ring_buffer {

    ($capacity:expr) => {
        use core::{mem, ptr};

        ///Automatically generated implementations of Ring buffer
        pub struct RingBuffer<T> {
            inner: mem::MaybeUninit<[T; $capacity]>,
            start: usize,
            end: usize,
        }

        impl<T> RingBuffer<T> {
            ///Capacity.
            pub const CAPACITY: usize = $capacity;

            ///Creates new instnace
            pub const fn new() -> Self {
                RingBuffer {
                    inner: mem::MaybeUninit::uninit(),
                    start: 0,
                    end: 0
                }
            }

            fn as_elem(&mut self, pos: usize) -> *const T {
                unsafe {
                    let ptr = self.inner.as_mut_ptr() as *const T;
                    ptr.offset(pos as isize)
                }
            }

            fn as_mut_elem(&mut self, pos: usize) -> *mut T {
                unsafe {
                    let ptr = self.inner.as_mut_ptr() as *mut T;
                    ptr.offset(pos as isize)
                }
            }

            ///Retrieves buffer capacity
            pub const fn capacity(&self) -> usize {
                Self::CAPACITY
            }

            ///Returns whether buffer is empty
            pub fn is_empty(&self) -> bool {
                self.start == self.end
            }

            ///Removes all elements from the buffer
            pub fn clear(&mut self) {
                while !self.is_empty() {
                    unsafe {
                        self.pop_unchecked();
                    }
                }
            }

            ///Adds new element.
            pub fn push(&mut self, value: T) {
                unsafe {
                    ptr::write(self.as_mut_elem(self.end), value);
                }
                self.end = (self.end + 1) % self.capacity();

                if self.start == self.end {
                    for _ in 0..10 {
                        drop(self.as_mut_elem(self.start));
                        self.start = (self.start + 1) % self.capacity();
                    }
                }
            }

            ///Pops element out of buffer.
            pub fn pop(&mut self) -> Option<T> {
                if self.start == self.end {
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

                ptr::copy_nonoverlapping(self.as_elem(self.start), result.as_mut_ptr(), 1);

                self.start = (self.start + 1) % self.capacity();

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
    fn test_queue() {
        let mut queue = RingBuffer::<usize>::new();
        assert_eq!(queue.capacity(), 512);
        assert!(queue.is_empty());

        for idx in 0..queue.capacity()+9 {
            queue.push(idx);
        }
        assert!(!queue.is_empty());

        for expected_item in 10..queue.capacity()+9 {
            let value = queue.pop().expect("Value");
            assert_eq!(value, expected_item);
        }

        assert!(queue.pop().is_none());
        assert!(queue.is_empty());

        for idx in 0..queue.capacity()+9 {
            queue.push(idx);
        }
        assert!(!queue.is_empty());

        queue.clear();
        assert!(queue.pop().is_none());
        assert!(queue.is_empty());
    }
}
