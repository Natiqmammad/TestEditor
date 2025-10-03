//! Pointer library for AFNS
//! 
//! This module provides pointer types including:
//! - Raw Pointers
//! - Smart Pointers
//! - Interior Mutability

use std::ptr;
use std::sync::{Arc, Mutex, RwLock};
use std::cell::{RefCell, Cell};
use std::rc::Rc;

/// Raw pointer wrapper
#[derive(Debug)]
pub struct AFNSRawPointer<T> {
    ptr: *mut T,
}

impl<T> AFNSRawPointer<T> {
    /// Create a new raw pointer
    pub fn new(value: T) -> Self {
        let boxed = Box::new(value);
        Self {
            ptr: Box::into_raw(boxed),
        }
    }

    /// Create a null pointer
    pub fn null() -> Self {
        Self {
            ptr: ptr::null_mut(),
        }
    }

    /// Check if the pointer is null
    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }

    /// Get the raw pointer
    pub fn as_ptr(&self) -> *mut T {
        self.ptr
    }

    /// Get a reference to the value
    pub unsafe fn as_ref(&self) -> Option<&T> {
        if self.ptr.is_null() {
            None
        } else {
            Some(&*self.ptr)
        }
    }

    /// Get a mutable reference to the value
    pub unsafe fn as_mut(&self) -> Option<&mut T> {
        if self.ptr.is_null() {
            None
        } else {
            Some(&mut *self.ptr)
        }
    }

    /// Drop the value and free memory
    pub unsafe fn drop(self) {
        if !self.ptr.is_null() {
            let _ = Box::from_raw(self.ptr);
        }
    }
}

impl<T> Clone for AFNSRawPointer<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T> Copy for AFNSRawPointer<T> {}

/// Smart pointer with reference counting
#[derive(Debug)]
pub struct AFNSSmartPointer<T> {
    inner: Rc<T>,
}

impl<T> AFNSSmartPointer<T> {
    /// Create a new smart pointer
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(value),
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

impl<T> Clone for AFNSSmartPointer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T> std::ops::Deref for AFNSSmartPointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

/// Atomic smart pointer with reference counting
#[derive(Debug)]
pub struct AFNSAtomicSmartPointer<T> {
    inner: Arc<T>,
}

impl<T> AFNSAtomicSmartPointer<T> {
    /// Create a new atomic smart pointer
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(value),
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

impl<T> Clone for AFNSAtomicSmartPointer<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> std::ops::Deref for AFNSAtomicSmartPointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}

/// Interior mutability with reference counting
#[derive(Debug)]
pub struct AFNSInteriorMutability<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> AFNSInteriorMutability<T> {
    /// Create a new interior mutability wrapper
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

impl<T> Clone for AFNSInteriorMutability<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

/// Atomic interior mutability with reference counting
#[derive(Debug)]
pub struct AFNSAtomicInteriorMutability<T> {
    inner: Arc<Mutex<T>>,
}

impl<T> AFNSAtomicInteriorMutability<T> {
    /// Create a new atomic interior mutability wrapper
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(value)),
        }
    }

    /// Get the reference count
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Get a reference to the inner value
    pub fn lock(&self) -> Result<std::sync::MutexGuard<T>, String> {
        self.inner.lock().map_err(|e| format!("Failed to lock: {}", e))
    }

    /// Try to get a reference to the inner value
    pub fn try_lock(&self) -> Result<std::sync::MutexGuard<T>, String> {
        self.inner.try_lock().map_err(|e| format!("Failed to try lock: {}", e))
    }
}

impl<T> Clone for AFNSAtomicInteriorMutability<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Cell for interior mutability
#[derive(Debug)]
pub struct AFNSCell<T> {
    inner: Box<Cell<T>>,
}

impl<T> AFNSCell<T> {
    /// Create a new cell
    pub fn new(value: T) -> Self {
        Self {
            inner: Box::new(Cell::new(value)),
        }
    }

    /// Get the value
    pub fn get(&self) -> T
    where
        T: Copy,
    {
        self.inner.get()
    }

    /// Set the value
    pub fn set(&self, value: T) {
        self.inner.set(value);
    }

    /// Replace the value and return the old value
    pub fn replace(&self, value: T) -> T {
        self.inner.replace(value)
    }

    /// Take the value and replace it with the default
    pub fn take(&self) -> T
    where
        T: Default,
    {
        self.inner.take()
    }
}

impl<T> Clone for AFNSCell<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

/// Read-write lock for interior mutability
#[derive(Debug)]
pub struct AFNSRwLock<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> AFNSRwLock<T> {
    /// Create a new read-write lock
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
        }
    }

    /// Get the reference count
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.inner)
    }

    /// Acquire a read lock
    pub fn read(&self) -> Result<std::sync::RwLockReadGuard<T>, String> {
        self.inner.read().map_err(|e| format!("Failed to acquire read lock: {}", e))
    }

    /// Acquire a write lock
    pub fn write(&self) -> Result<std::sync::RwLockWriteGuard<T>, String> {
        self.inner.write().map_err(|e| format!("Failed to acquire write lock: {}", e))
    }

    /// Try to acquire a read lock
    pub fn try_read(&self) -> Result<std::sync::RwLockReadGuard<T>, String> {
        self.inner.try_read().map_err(|e| format!("Failed to try read lock: {}", e))
    }

    /// Try to acquire a write lock
    pub fn try_write(&self) -> Result<std::sync::RwLockWriteGuard<T>, String> {
        self.inner.try_write().map_err(|e| format!("Failed to try write lock: {}", e))
    }
}

impl<T> Clone for AFNSRwLock<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

// Type aliases for common use cases
pub type RawPointer<T> = AFNSRawPointer<T>;
pub type SmartPointer<T> = AFNSSmartPointer<T>;
pub type AtomicSmartPointer<T> = AFNSAtomicSmartPointer<T>;
pub type InteriorMutability<T> = AFNSInteriorMutability<T>;
pub type AtomicInteriorMutability<T> = AFNSAtomicInteriorMutability<T>;
pub type Cell<T> = AFNSCell<T>;
pub type RwLock<T> = AFNSRwLock<T>;