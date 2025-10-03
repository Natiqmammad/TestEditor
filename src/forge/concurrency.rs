//! Concurrency library for AFNS
//! 
//! This module provides concurrency primitives including:
//! - Thread: Thread management
//! - Mutex: Mutual exclusion
//! - RwLock: Read-write lock
//! - Channel: Message passing
//! - Atomic: Atomic operations

use std::sync::{Arc, Mutex, RwLock, mpsc};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use std::thread;
use std::time::Duration;

/// Thread management
#[derive(Debug)]
pub struct AFNSThread {
    handle: Option<thread::JoinHandle<()>>,
    id: String,
    name: Option<String>,
}

impl AFNSThread {
    /// Create a new thread
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let id = uuid::Uuid::new_v4().to_string();
        let handle = thread::spawn(f);
        
        Self {
            handle: Some(handle),
            id,
            name: None,
        }
    }

    /// Create a new thread with a name
    pub fn new_with_name<F>(name: String, f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let id = uuid::Uuid::new_v4().to_string();
        let handle = thread::Builder::new()
            .name(name.clone())
            .spawn(f)
            .unwrap();
        
        Self {
            handle: Some(handle),
            id,
            name: Some(name),
        }
    }

    /// Join the thread
    pub fn join(self) -> Result<(), String> {
        if let Some(handle) = self.handle {
            handle.join().map_err(|_| "Thread panicked".to_string())
        } else {
            Err("Thread already joined".to_string())
        }
    }

    /// Get the thread ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the thread name
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Check if the thread is finished
    pub fn is_finished(&self) -> bool {
        self.handle.is_none()
    }
}

/// Mutual exclusion
#[derive(Debug)]
pub struct AFNSMutex<T> {
    inner: Arc<Mutex<T>>,
}

impl<T> AFNSMutex<T> {
    /// Create a new mutex
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(data)),
        }
    }

    /// Lock the mutex
    pub fn lock(&self) -> Result<AFNSMutexGuard<T>, String> {
        self.inner.lock()
            .map(|guard| AFNSMutexGuard { guard })
            .map_err(|_| "Mutex lock failed".to_string())
    }

    /// Try to lock the mutex
    pub fn try_lock(&self) -> Result<AFNSMutexGuard<T>, String> {
        self.inner.try_lock()
            .map(|guard| AFNSMutexGuard { guard })
            .map_err(|_| "Mutex try_lock failed".to_string())
    }

    /// Clone the mutex
    pub fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Clone for AFNSMutex<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Mutex guard
pub struct AFNSMutexGuard<T> {
    guard: std::sync::MutexGuard<'static, T>,
}

impl<T> std::ops::Deref for AFNSMutexGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

impl<T> std::ops::DerefMut for AFNSMutexGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.guard
    }
}

/// Read-write lock
#[derive(Debug)]
pub struct AFNSRwLock<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> AFNSRwLock<T> {
    /// Create a new read-write lock
    pub fn new(data: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(data)),
        }
    }

    /// Acquire a read lock
    pub fn read(&self) -> Result<AFNSRwLockReadGuard<T>, String> {
        self.inner.read()
            .map(|guard| AFNSRwLockReadGuard { guard })
            .map_err(|_| "RwLock read lock failed".to_string())
    }

    /// Acquire a write lock
    pub fn write(&self) -> Result<AFNSRwLockWriteGuard<T>, String> {
        self.inner.write()
            .map(|guard| AFNSRwLockWriteGuard { guard })
            .map_err(|_| "RwLock write lock failed".to_string())
    }

    /// Try to acquire a read lock
    pub fn try_read(&self) -> Result<AFNSRwLockReadGuard<T>, String> {
        self.inner.try_read()
            .map(|guard| AFNSRwLockReadGuard { guard })
            .map_err(|_| "RwLock try_read failed".to_string())
    }

    /// Try to acquire a write lock
    pub fn try_write(&self) -> Result<AFNSRwLockWriteGuard<T>, String> {
        self.inner.try_write()
            .map(|guard| AFNSRwLockWriteGuard { guard })
            .map_err(|_| "RwLock try_write failed".to_string())
    }

    /// Clone the read-write lock
    pub fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Clone for AFNSRwLock<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// Read-write lock read guard
pub struct AFNSRwLockReadGuard<T> {
    guard: std::sync::RwLockReadGuard<'static, T>,
}

impl<T> std::ops::Deref for AFNSRwLockReadGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

/// Read-write lock write guard
pub struct AFNSRwLockWriteGuard<T> {
    guard: std::sync::RwLockWriteGuard<'static, T>,
}

impl<T> std::ops::Deref for AFNSRwLockWriteGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.guard
    }
}

impl<T> std::ops::DerefMut for AFNSRwLockWriteGuard<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.guard
    }
}

/// Message passing channel
#[derive(Debug)]
pub struct AFNSChannel<T> {
    sender: mpsc::Sender<T>,
    receiver: mpsc::Receiver<T>,
}

impl<T> AFNSChannel<T> {
    /// Create a new channel
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver }
    }

    /// Create a new channel with a buffer
    pub fn new_bounded(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::sync_channel(capacity);
        Self { sender, receiver }
    }

    /// Send a message
    pub fn send(&self, message: T) -> Result<(), String> {
        self.sender.send(message)
            .map_err(|_| "Channel send failed".to_string())
    }

    /// Receive a message
    pub fn recv(&self) -> Result<T, String> {
        self.receiver.recv()
            .map_err(|_| "Channel receive failed".to_string())
    }

    /// Try to receive a message
    pub fn try_recv(&self) -> Result<T, String> {
        self.receiver.try_recv()
            .map_err(|e| format!("Channel try_receive failed: {:?}", e))
    }

    /// Receive a message with timeout
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, String> {
        self.receiver.recv_timeout(timeout)
            .map_err(|e| format!("Channel receive timeout: {:?}", e))
    }

    /// Clone the sender
    pub fn clone_sender(&self) -> AFNSChannelSender<T> {
        AFNSChannelSender {
            sender: self.sender.clone(),
        }
    }

    /// Get the receiver
    pub fn receiver(&self) -> AFNSChannelReceiver<T> {
        AFNSChannelReceiver {
            receiver: &self.receiver,
        }
    }
}

/// Channel sender
#[derive(Debug, Clone)]
pub struct AFNSChannelSender<T> {
    sender: mpsc::Sender<T>,
}

impl<T> AFNSChannelSender<T> {
    /// Send a message
    pub fn send(&self, message: T) -> Result<(), String> {
        self.sender.send(message)
            .map_err(|_| "Channel send failed".to_string())
    }
}

/// Channel receiver
#[derive(Debug)]
pub struct AFNSChannelReceiver<'a, T> {
    receiver: &'a mpsc::Receiver<T>,
}

impl<'a, T> AFNSChannelReceiver<'a, T> {
    /// Receive a message
    pub fn recv(&self) -> Result<T, String> {
        self.receiver.recv()
            .map_err(|_| "Channel receive failed".to_string())
    }

    /// Try to receive a message
    pub fn try_recv(&self) -> Result<T, String> {
        self.receiver.try_recv()
            .map_err(|e| format!("Channel try_receive failed: {:?}", e))
    }

    /// Receive a message with timeout
    pub fn recv_timeout(&self, timeout: Duration) -> Result<T, String> {
        self.receiver.recv_timeout(timeout)
            .map_err(|e| format!("Channel receive timeout: {:?}", e))
    }
}

/// Atomic boolean
#[derive(Debug)]
pub struct AFNSAtomicBool {
    inner: Box<AtomicBool>,
}

impl AFNSAtomicBool {
    /// Create a new atomic boolean
    pub fn new(value: bool) -> Self {
        Self {
            inner: Box::new(AtomicBool::new(value)),
        }
    }

    /// Load the value
    pub fn load(&self, order: Ordering) -> bool {
        self.inner.load(order)
    }

    /// Store a value
    pub fn store(&self, value: bool, order: Ordering) {
        self.inner.store(value, order);
    }

    /// Compare and swap
    pub fn compare_and_swap(&self, current: bool, new: bool, order: Ordering) -> bool {
        self.inner.compare_and_swap(current, new, order)
    }

    /// Swap the value
    pub fn swap(&self, value: bool, order: Ordering) -> bool {
        self.inner.swap(value, order)
    }
}

/// Atomic integer
#[derive(Debug)]
pub struct AFNSAtomicI32 {
    inner: Box<AtomicI32>,
}

impl AFNSAtomicI32 {
    /// Create a new atomic integer
    pub fn new(value: i32) -> Self {
        Self {
            inner: Box::new(AtomicI32::new(value)),
        }
    }

    /// Load the value
    pub fn load(&self, order: Ordering) -> i32 {
        self.inner.load(order)
    }

    /// Store a value
    pub fn store(&self, value: i32, order: Ordering) {
        self.inner.store(value, order);
    }

    /// Compare and swap
    pub fn compare_and_swap(&self, current: i32, new: i32, order: Ordering) -> i32 {
        self.inner.compare_and_swap(current, new, order)
    }

    /// Swap the value
    pub fn swap(&self, value: i32, order: Ordering) -> i32 {
        self.inner.swap(value, order)
    }

    /// Add to the value
    pub fn add(&self, value: i32, order: Ordering) -> i32 {
        self.inner.fetch_add(value, order)
    }

    /// Subtract from the value
    pub fn sub(&self, value: i32, order: Ordering) -> i32 {
        self.inner.fetch_sub(value, order)
    }

    /// Increment the value
    pub fn inc(&self, order: Ordering) -> i32 {
        self.inner.fetch_add(1, order)
    }

    /// Decrement the value
    pub fn dec(&self, order: Ordering) -> i32 {
        self.inner.fetch_sub(1, order)
    }
}

/// Atomic unsigned integer
#[derive(Debug)]
pub struct AFNSAtomicU32 {
    inner: Box<AtomicU32>,
}

impl AFNSAtomicU32 {
    /// Create a new atomic unsigned integer
    pub fn new(value: u32) -> Self {
        Self {
            inner: Box::new(AtomicU32::new(value)),
        }
    }

    /// Load the value
    pub fn load(&self, order: Ordering) -> u32 {
        self.inner.load(order)
    }

    /// Store a value
    pub fn store(&self, value: u32, order: Ordering) {
        self.inner.store(value, order);
    }

    /// Compare and swap
    pub fn compare_and_swap(&self, current: u32, new: u32, order: Ordering) -> u32 {
        self.inner.compare_and_swap(current, new, order)
    }

    /// Swap the value
    pub fn swap(&self, value: u32, order: Ordering) -> u32 {
        self.inner.swap(value, order)
    }

    /// Add to the value
    pub fn add(&self, value: u32, order: Ordering) -> u32 {
        self.inner.fetch_add(value, order)
    }

    /// Subtract from the value
    pub fn sub(&self, value: u32, order: Ordering) -> u32 {
        self.inner.fetch_sub(value, order)
    }

    /// Increment the value
    pub fn inc(&self, order: Ordering) -> u32 {
        self.inner.fetch_add(1, order)
    }

    /// Decrement the value
    pub fn dec(&self, order: Ordering) -> u32 {
        self.inner.fetch_sub(1, order)
    }
}

// Type aliases for common use cases
pub type Thread = AFNSThread;
pub type Mutex<T> = AFNSMutex<T>;
pub type RwLock<T> = AFNSRwLock<T>;
pub type Channel<T> = AFNSChannel<T>;
pub type AtomicBool = AFNSAtomicBool;
pub type AtomicI32 = AFNSAtomicI32;
pub type AtomicU32 = AFNSAtomicU32;