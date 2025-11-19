//! Memory library for AFNS
//!
//! This module provides memory management including:
//! - Memory Allocation
//! - Management
//! - Reference Counting

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Memory allocator statistics
#[derive(Debug, Default)]
pub struct AFNSMemoryStats {
    allocated: AtomicUsize,
    deallocated: AtomicUsize,
    peak: AtomicUsize,
}

impl AFNSMemoryStats {
    /// Get the current allocated memory
    pub fn allocated(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    /// Get the total deallocated memory
    pub fn deallocated(&self) -> usize {
        self.deallocated.load(Ordering::Relaxed)
    }

    /// Get the peak memory usage
    pub fn peak(&self) -> usize {
        self.peak.load(Ordering::Relaxed)
    }

    /// Get the current memory usage
    pub fn current(&self) -> usize {
        self.allocated() - self.deallocated()
    }

    /// Update the peak memory usage
    fn update_peak(&self, current: usize) {
        let mut peak = self.peak.load(Ordering::Relaxed);
        loop {
            if current <= peak {
                break;
            }
            match self.peak.compare_exchange_weak(
                peak,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_peak) => peak = new_peak,
            }
        }
    }
}

/// Custom memory allocator with statistics
pub struct AFNSAllocator {
    stats: AFNSMemoryStats,
}

impl AFNSAllocator {
    /// Create a new allocator
    pub fn new() -> Self {
        Self {
            stats: AFNSMemoryStats::default(),
        }
    }

    /// Get memory statistics
    pub fn stats(&self) -> &AFNSMemoryStats {
        &self.stats
    }
}

unsafe impl GlobalAlloc for AFNSAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let ptr = System.alloc(layout);

        if !ptr.is_null() {
            self.stats.allocated.fetch_add(size, Ordering::Relaxed);
            self.stats.update_peak(self.stats.current());
        }

        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        System.dealloc(ptr, layout);
        self.stats.deallocated.fetch_add(size, Ordering::Relaxed);
    }
}

/// Reference counted pointer
#[derive(Debug)]
pub struct AFNSRc<T> {
    inner: Box<Rc<T>>,
}

impl<T> AFNSRc<T> {
    /// Create a new reference counted pointer
    pub fn new(value: T) -> Self {
        Self {
            inner: Box::new(Rc::new(value)),
        }
    }

    /// Get the reference count
    pub fn strong_count(&self) -> usize {
        Rc::strong_count(&self.inner)
    }

    /// Check if this is the only reference
    pub fn is_unique(&self) -> bool {
        Rc::is_unique(&self.inner)
    }

    /// Get a reference to the inner value
    pub fn as_ref(&self) -> &T {
        self.inner.as_ref()
    }

    /// Try to unwrap the value if this is the only reference
    pub fn try_unwrap(self) -> Result<T, Self> {
        Rc::try_unwrap(self.inner).map_err(|inner| Self { inner })
    }
}

impl<T> Clone for AFNSRc<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T> std::ops::Deref for AFNSRc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

/// Atomic reference counted pointer
#[derive(Debug)]
pub struct AFNSArc<T> {
    inner: Box<Arc<T>>,
}

impl<T> AFNSArc<T> {
    /// Create a new atomic reference counted pointer
    pub fn new(value: T) -> Self {
        Self {
            inner: Box::new(Arc::new(value)),
        }
    }

    /// Get the reference count
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Check if this is the only reference
    pub fn is_unique(&self) -> bool {
        Arc::strong_count(&self.inner) == 1
    }

    /// Get a reference to the inner value
    pub fn as_ref(&self) -> &T {
        self.inner.as_ref()
    }

    /// Try to unwrap the value if this is the only reference
    pub fn try_unwrap(self) -> Result<T, Self> {
        Arc::try_unwrap(self.inner).map_err(|inner| Self { inner })
    }
}

impl<T> Clone for AFNSArc<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> std::ops::Deref for AFNSArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

/// Reference counted cell for interior mutability
#[derive(Debug)]
pub struct AFNSRcCell<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> AFNSRcCell<T> {
    /// Create a new reference counted cell
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(value)),
        }
    }

    /// Get the reference count
    pub fn strong_count(&self) -> usize {
        Rc::strong_count(&self.inner)
    }

    /// Get a reference to the inner value
    pub fn borrow(&self) -> std::cell::Ref<T> {
        self.inner.borrow()
    }

    /// Get a mutable reference to the inner value
    pub fn borrow_mut(&self) -> std::cell::RefMut<T> {
        self.inner.borrow_mut()
    }

    /// Try to get a reference to the inner value
    pub fn try_borrow(&self) -> Result<std::cell::Ref<T>, std::cell::BorrowError> {
        self.inner.try_borrow()
    }

    /// Try to get a mutable reference to the inner value
    pub fn try_borrow_mut(&self) -> Result<std::cell::RefMut<T>, std::cell::BorrowMutError> {
        self.inner.try_borrow_mut()
    }
}

impl<T> Clone for AFNSRcCell<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

/// Atomic reference counted cell for interior mutability
#[derive(Debug)]
pub struct AFNSArcCell<T> {
    inner: Arc<std::sync::Mutex<T>>,
}

impl<T> AFNSArcCell<T> {
    /// Create a new atomic reference counted cell
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(std::sync::Mutex::new(value)),
        }
    }

    /// Get the reference count
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Get a reference to the inner value
    pub fn lock(&self) -> Result<std::sync::MutexGuard<T>, String> {
        self.inner
            .lock()
            .map_err(|e| format!("Failed to lock: {}", e))
    }

    /// Try to get a reference to the inner value
    pub fn try_lock(&self) -> Result<std::sync::MutexGuard<T>, String> {
        self.inner
            .try_lock()
            .map_err(|e| format!("Failed to try lock: {}", e))
    }
}

impl<T> Clone for AFNSArcCell<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Memory pool for efficient allocation
#[derive(Debug)]
pub struct AFNSMemoryPool<T> {
    pool: Vec<T>,
    available: Vec<usize>,
    next_id: usize,
}

impl<T> AFNSMemoryPool<T> {
    /// Create a new memory pool
    pub fn new(capacity: usize) -> Self {
        Self {
            pool: Vec::with_capacity(capacity),
            available: Vec::new(),
            next_id: 0,
        }
    }

    /// Allocate an item from the pool
    pub fn allocate(&mut self, item: T) -> Result<usize, String> {
        if let Some(index) = self.available.pop() {
            self.pool[index] = item;
            Ok(index)
        } else if self.pool.len() < self.pool.capacity() {
            let index = self.pool.len();
            self.pool.push(item);
            Ok(index)
        } else {
            Err("Memory pool is full".to_string())
        }
    }

    /// Deallocate an item from the pool
    pub fn deallocate(&mut self, index: usize) -> Result<(), String> {
        if index < self.pool.len() {
            self.available.push(index);
            Ok(())
        } else {
            Err("Invalid index".to_string())
        }
    }

    /// Get an item from the pool
    pub fn get(&self, index: usize) -> Option<&T> {
        self.pool.get(index)
    }

    /// Get a mutable reference to an item from the pool
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.pool.get_mut(index)
    }

    /// Get the pool size
    pub fn size(&self) -> usize {
        self.pool.len()
    }

    /// Get the available count
    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    /// Get the capacity
    pub fn capacity(&self) -> usize {
        self.pool.capacity()
    }
}

// Type aliases for common use cases
pub type MemoryStats = AFNSMemoryStats;
pub type Allocator = AFNSAllocator;
pub type Rc<T> = AFNSRc<T>;
pub type Arc<T> = AFNSArc<T>;
pub type RcCell<T> = AFNSRcCell<T>;
pub type ArcCell<T> = AFNSArcCell<T>;
pub type MemoryPool<T> = AFNSMemoryPool<T>;
