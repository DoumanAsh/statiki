//!Ring buffer implementation

use core::{fmt, mem};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

struct Assert<const C: usize>;

impl<const C: usize> Assert<C> {
    const RING_BUFFER_GOOD_CAPACITY: () = {
        assert!(C != 0, "Capacity cannot be 0");
        assert!((C & (C - 1)) == 0, "Capacity is not power of 2");
    };
}

///Atomic ring buffer
///
///Based on <https://www.codeproject.com/Articles/43510/Lock-Free-Single-Producer-Single-Consumer-Circular>
///
///When used directly, all operations are performed with relaxed ordering
pub struct RingBuffer<T, const C: usize> {
    inner: [UnsafeCell<mem::MaybeUninit<T>>; C],
    read: AtomicUsize,
    write: AtomicUsize,
}

impl<T, const CAPACITY: usize> RingBuffer<T, CAPACITY> {
    const INIT: UnsafeCell<mem::MaybeUninit<T>> = UnsafeCell::new(mem::MaybeUninit::uninit());

    #[inline(always)]
    ///Creates new instance
    pub const fn new() -> Self {
        let _ = Assert::<CAPACITY>::RING_BUFFER_GOOD_CAPACITY;

        Self {
            inner: [Self::INIT; CAPACITY],
            read: AtomicUsize::new(0),
            write: AtomicUsize::new(0),
        }
    }

    #[inline(always)]
    const fn mask_idx(idx: usize) -> usize {
        idx & (CAPACITY - 1)
    }

    #[inline(always)]
    ///Retrieves buffer capacity.
    pub const fn capacity(&self) -> usize {
        CAPACITY
    }

    #[inline(always)]
    ///Returns the number of elements in buffer.
    pub fn size(&self) -> usize {
        self.write.load(Ordering::Relaxed).wrapping_sub(self.read.load(Ordering::Relaxed))
    }

    #[inline(always)]
    ///Returns whether buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.write.load(Ordering::Relaxed) == self.read.load(Ordering::Relaxed)
    }

    #[inline(always)]
    ///Returns whether buffer is empty.
    pub fn is_full(&self) -> bool {
        self.size() == CAPACITY
    }

    ///Adds new element, unconditionally overriding last unread element
    pub fn push(&mut self, value: T) {
        let write = self.write.fetch_add(1, Ordering::Relaxed);
        let read = self.read.load(Ordering::Relaxed);
        let remaning = write.wrapping_sub(read);

        if remaning == CAPACITY {
            unsafe {
                (self.inner.get_unchecked(Self::mask_idx(read)).get() as *const T).read();
            }
            self.read.store(read.wrapping_add(1), Ordering::Relaxed);
        }

        unsafe {
            self.inner.get_unchecked(Self::mask_idx(write)).get().write(mem::MaybeUninit::new(value))
        }
    }

    #[inline]
    ///Attempts to push element onto buffer.
    ///
    ///In case of buffer being full, returns `value` otherwise `None` and element is added to the buffer
    pub fn try_push(&mut self, value: T) -> Option<T> {
        //self.inner_push(value, Ordering::Acquire, Ordering::Release)
        self.inner_push(value, Ordering::Relaxed, Ordering::Relaxed)
    }

    #[inline]
    ///Attempts to push element onto buffer.
    ///
    ///In case of buffer being full, returns `value` otherwise `None` and element is added to the buffer
    fn inner_push(&self, value: T, read_op: Ordering, write_op: Ordering) -> Option<T> {
        let idx = self.write.load(Ordering::Relaxed);
        let remaning = idx.wrapping_sub(self.read.load(read_op));

        if remaning != CAPACITY {
            unsafe {
                self.inner.get_unchecked(Self::mask_idx(idx)).get().write(mem::MaybeUninit::new(value))
            }
            self.write.store(idx.wrapping_add(1), write_op);
            None
        } else {
            Some(value)
        }
    }

    #[inline]
    ///Unconditionally pushes element onto buffer.
    pub unsafe fn push_unchecked(&mut self, value: T) {
        let idx = Self::mask_idx(
            self.write.fetch_add(1, Ordering::Relaxed)
        );

        self.inner.get_unchecked(idx).get().write(mem::MaybeUninit::new(value))
    }

    #[inline(always)]
    ///Attempts to retrieve element from buffer.
    pub fn pop(&mut self) -> Option<T> {
        //self.inner_pop(Ordering::Acquire, Ordering::Release)
        self.inner_pop(Ordering::Relaxed, Ordering::Relaxed)
    }

    #[inline]
    ///Attempts to retrieve element from buffer.
    fn inner_pop(&self, write_op: Ordering, read_op: Ordering) -> Option<T> {
        let idx = self.read.load(Ordering::Relaxed);

        if idx != self.write.load(write_op) {
            let value = unsafe {
                (self.inner.get_unchecked(Self::mask_idx(idx)).get() as *const T).read()
            };
            self.read.store(idx.wrapping_add(1), read_op);
            Some(value)
        } else {
            None
        }
    }

    #[inline]
    ///Unconditionally retrieves element from buffer.
    pub unsafe fn pop_unchecked(&mut self) -> T {
        let idx = Self::mask_idx(
            self.read.fetch_add(1, Ordering::Relaxed)
        );

        (self.inner.get_unchecked(idx).get() as *const T).read()
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
            self.read.store(self.write.load(Ordering::Relaxed), Ordering::Relaxed);
        }
    }

    #[inline(always)]
    ///Splits into thread safe producer and consumer
    pub fn split(&mut self) -> (Producer<'_, T, CAPACITY>, Consumer<'_, T, CAPACITY>) {
        (Producer(self), Consumer(self))
    }
}

impl<T, const CAPACITY: usize> Drop for RingBuffer<T, CAPACITY> {
    #[inline(always)]
    fn drop(&mut self) {
        self.clear();
    }
}

impl<T, const CAPACITY: usize> Default for RingBuffer<T, CAPACITY> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const CAPACITY: usize> fmt::Debug for RingBuffer<T, CAPACITY> {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("RingBuffer")
           .field("capacity", &CAPACITY)
           .field("size", &self.size())
           .finish()
    }
}

///Consumer of Ring RingBuffer
///
///As name implies, it is only capable of consuming elements from buffer
pub struct Consumer<'a, T, const N: usize>(&'a RingBuffer<T, N>);

impl<'a, T, const N: usize> Consumer<'a, T, N> {
    #[inline(always)]
    ///Attempts to retrieve element from buffer.
    pub fn pop(&self) -> Option<T> {
        self.0.inner_pop(Ordering::Acquire, Ordering::Release)
    }
}

unsafe impl<'a, T, const N: usize> Sync for Consumer<'a, T, N> {
}
unsafe impl<'a, T, const N: usize> Send for Consumer<'a, T, N> {
}

///Producer of Ring RingBuffer
///
///As name implies, it is only capable of putting elements onto buffer
pub struct Producer<'a, T, const N: usize>(&'a RingBuffer<T, N>);

impl<'a, T, const N: usize> Producer<'a, T, N> {
    #[inline]
    ///Attempts to push element onto buffer.
    ///
    ///In case of buffer being full, returns `value` otherwise `None` and element is added to the buffer
    pub fn try_push(&self, value: T) -> Option<T> {
        self.0.inner_push(value, Ordering::Acquire, Ordering::Release)
    }
}

unsafe impl<'a, T, const N: usize> Sync for Producer<'a, T, N> {
}
unsafe impl<'a, T, const N: usize> Send for Producer<'a, T, N> {
}
