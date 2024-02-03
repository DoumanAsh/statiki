use core::{mem, ptr};

struct Assert<const C: usize>;

impl<const C: usize> Assert<C> {
    const RING_BUFFER_GOOD_CAPACITY: () = {
        assert!(C != 0, "Capacity cannot be 0");
        assert!((C & (C - 1)) == 0, "Capacity is not power of 2");
    };
}

///Ring buffer
pub struct RingBuffer<T, const C: usize> {
    inner: mem::MaybeUninit<[mem::MaybeUninit<T>; C]>,
    read: usize,
    write: usize,
}

impl<T, const CAPACITY: usize> RingBuffer<T, CAPACITY> {
    #[inline(always)]
    ///Creates new instance
    pub const fn new() -> Self {
        let _ = Assert::<CAPACITY>::RING_BUFFER_GOOD_CAPACITY;

        RingBuffer {
            inner: mem::MaybeUninit::uninit(),
            read: 0,
            write: 0,
        }
    }

    #[inline(always)]
    fn as_elem(&self, pos: usize) -> *const T {
        unsafe {
            let ptr = self.inner.as_ptr() as *const T;
            ptr.add(self.mask_idx(pos))
        }
    }

    #[inline(always)]
    fn as_mut_elem(&mut self, pos: usize) -> *mut T {
        unsafe {
            let ptr = self.inner.as_mut_ptr() as *mut T;
            ptr.add(self.mask_idx(pos))
        }
    }

    #[inline(always)]
    ///Returns the number of elements in buffer.
    pub const fn size(&self) -> usize {
        self.write - self.read
    }

    #[inline(always)]
    ///Retrieves buffer capacity.
    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    #[inline(always)]
    const fn mask_idx(&self, idx: usize) -> usize {
        idx & (CAPACITY - 1)
    }

    #[inline(always)]
    ///Returns whether buffer is empty.
    pub const fn is_empty(&self) -> bool {
        self.write == self.read
    }

    #[inline(always)]
    ///Returns whether buffer is empty.
    pub const fn is_full(&self) -> bool {
        self.size() == self.capacity()
    }

    #[inline(always)]
    ///Removes all elements from the buffer.
    pub fn clear(&mut self) {
        if mem::needs_drop::<T>() {
            for _ in 0..self.size() {
                unsafe {
                    self.pop_unchecked();
                }
            }
        } else {
            self.read = self.write;
        }
    }

    ///Adds new element.
    pub fn push(&mut self, value: T) {
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

impl<T, const C: usize> Drop for RingBuffer<T, C> {
    #[inline(always)]
    fn drop(&mut self) {
        self.clear();
    }
}
