//! Collections library for AFNS
//!
//! This module provides comprehensive collection types including:
//! - Array: Dynamic array with type safety
//! - Map: Key-value mapping with various key types
//! - Set: Unordered collection of unique elements
//! - Queue: First-in-first-out data structure
//! - Stack: Last-in-first-out data structure
//! - LinkedList: Doubly linked list
//! - RingBuffer: Circular buffer with fixed capacity
//! - CircularBuffer: Alias for RingBuffer

use std::collections::hash_map::Iter as HashMapIter;
use std::collections::hash_set::Iter as HashSetIter;
use std::collections::linked_list::Iter as LinkedListIter;
use std::collections::vec_deque::Iter as VecDequeIter;
use std::collections::{HashMap, HashSet, LinkedList as StdLinkedList, VecDeque};
use std::fmt;
use std::ops::{Index, IndexMut};

/// Dynamic array with type safety
#[derive(Debug, Clone)]
pub struct AFNSArray<T> {
    data: Vec<T>,
}

impl<T> AFNSArray<T> {
    /// Create a new empty array
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Create a new array with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Create a new array with initial values
    pub fn from_vec(data: Vec<T>) -> Self {
        Self { data }
    }

    /// Get the length of the array
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the array is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the capacity of the array
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Reserve additional capacity
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    /// Shrink the capacity to fit the current length
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
    }

    /// Push an element to the end
    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    /// Pop an element from the end
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    /// Insert an element at the specified index
    pub fn insert(&mut self, index: usize, value: T) -> Result<(), String> {
        if index > self.data.len() {
            return Err(format!(
                "Index {} out of bounds for array of length {}",
                index,
                self.data.len()
            ));
        }
        self.data.insert(index, value);
        Ok(())
    }

    /// Remove an element at the specified index
    pub fn remove(&mut self, index: usize) -> Result<T, String> {
        if index >= self.data.len() {
            return Err(format!(
                "Index {} out of bounds for array of length {}",
                index,
                self.data.len()
            ));
        }
        Ok(self.data.remove(index))
    }

    /// Get an element at the specified index
    pub fn get(&self, index: usize) -> Option<&T> {
        self.data.get(index)
    }

    /// Get a mutable reference to an element at the specified index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.data.get_mut(index)
    }

    /// Get the first element
    pub fn first(&self) -> Option<&T> {
        self.data.first()
    }

    /// Get the last element
    pub fn last(&self) -> Option<&T> {
        self.data.last()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Reverse the array in place
    pub fn reverse(&mut self) {
        self.data.reverse();
    }

    /// Sort the array in place
    pub fn sort(&mut self)
    where
        T: Ord,
    {
        self.data.sort();
    }

    /// Sort the array with a custom comparison function
    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering,
    {
        self.data.sort_by(compare);
    }

    /// Binary search for an element
    pub fn binary_search(&self, x: &T) -> Result<usize, usize>
    where
        T: Ord,
    {
        self.data.binary_search(x)
    }

    /// Check if the array contains an element
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq,
    {
        self.data.contains(x)
    }

    /// Find the index of an element
    pub fn position(&self, x: &T) -> Option<usize>
    where
        T: PartialEq,
    {
        self.data.iter().position(|item| item == x)
    }

    /// Convert to a vector
    pub fn into_vec(self) -> Vec<T> {
        self.data
    }

    /// Get an iterator over the elements
    pub fn iter(&self) -> std::slice::Iter<T> {
        self.data.iter()
    }

    /// Get a mutable iterator over the elements
    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.data.iter_mut()
    }
}

impl<T> Index<usize> for AFNSArray<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T> IndexMut<usize> for AFNSArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T> fmt::Display for AFNSArray<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, item) in self.data.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, "]")
    }
}

/// Key-value mapping with various key types
#[derive(Debug, Clone)]
pub struct AFNSMap<K, V> {
    data: HashMap<K, V>,
}

impl<K, V> AFNSMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    /// Create a new empty map
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Create a new map with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
        }
    }

    /// Get the number of key-value pairs
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the map is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the capacity of the map
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Reserve additional capacity
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    /// Shrink the capacity to fit the current size
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
    }

    /// Insert a key-value pair
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.data.insert(key, value)
    }

    /// Remove a key-value pair
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.data.remove(key)
    }

    /// Get a value by key
    pub fn get(&self, key: &K) -> Option<&V> {
        self.data.get(key)
    }

    /// Get a mutable reference to a value by key
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.data.get_mut(key)
    }

    /// Check if the map contains a key
    pub fn contains_key(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }

    /// Get all keys
    pub fn keys(&self) -> std::collections::hash_map::Keys<K, V> {
        self.data.keys()
    }

    /// Get all values
    pub fn values(&self) -> std::collections::hash_map::Values<K, V> {
        self.data.values()
    }

    /// Get all values mutably
    pub fn values_mut(&mut self) -> std::collections::hash_map::ValuesMut<K, V> {
        self.data.values_mut()
    }

    /// Clear all key-value pairs
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get an iterator over key-value pairs
    pub fn iter(&self) -> HashMapIter<K, V> {
        self.data.iter()
    }

    /// Get a mutable iterator over key-value pairs
    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<K, V> {
        self.data.iter_mut()
    }
}

/// Unordered collection of unique elements
#[derive(Debug, Clone)]
pub struct AFNSSet<T> {
    data: HashSet<T>,
}

impl<T> AFNSSet<T>
where
    T: std::hash::Hash + Eq + Clone,
{
    /// Create a new empty set
    pub fn new() -> Self {
        Self {
            data: HashSet::new(),
        }
    }

    /// Create a new set with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashSet::with_capacity(capacity),
        }
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the set is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the capacity of the set
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Reserve additional capacity
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    /// Shrink the capacity to fit the current size
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
    }

    /// Insert an element
    pub fn insert(&mut self, value: T) -> bool {
        self.data.insert(value)
    }

    /// Remove an element
    pub fn remove(&mut self, value: &T) -> bool {
        self.data.remove(value)
    }

    /// Check if the set contains an element
    pub fn contains(&self, value: &T) -> bool {
        self.data.contains(value)
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get an iterator over the elements
    pub fn iter(&self) -> HashSetIter<T> {
        self.data.iter()
    }

    /// Union with another set
    pub fn union(&self, other: &AFNSSet<T>) -> AFNSSet<T> {
        Self {
            data: self.data.union(&other.data).cloned().collect(),
        }
    }

    /// Intersection with another set
    pub fn intersection(&self, other: &AFNSSet<T>) -> AFNSSet<T> {
        Self {
            data: self.data.intersection(&other.data).cloned().collect(),
        }
    }

    /// Difference with another set
    pub fn difference(&self, other: &AFNSSet<T>) -> AFNSSet<T> {
        Self {
            data: self.data.difference(&other.data).cloned().collect(),
        }
    }

    /// Symmetric difference with another set
    pub fn symmetric_difference(&self, other: &AFNSSet<T>) -> AFNSSet<T> {
        Self {
            data: self
                .data
                .symmetric_difference(&other.data)
                .cloned()
                .collect(),
        }
    }

    /// Check if this set is a subset of another set
    pub fn is_subset(&self, other: &AFNSSet<T>) -> bool {
        self.data.is_subset(&other.data)
    }

    /// Check if this set is a superset of another set
    pub fn is_superset(&self, other: &AFNSSet<T>) -> bool {
        self.data.is_superset(&other.data)
    }

    /// Check if this set is disjoint from another set
    pub fn is_disjoint(&self, other: &AFNSSet<T>) -> bool {
        self.data.is_disjoint(&other.data)
    }
}

/// First-in-first-out data structure
#[derive(Debug, Clone)]
pub struct AFNSQueue<T> {
    data: VecDeque<T>,
}

impl<T> AFNSQueue<T> {
    /// Create a new empty queue
    pub fn new() -> Self {
        Self {
            data: VecDeque::new(),
        }
    }

    /// Create a new queue with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: VecDeque::with_capacity(capacity),
        }
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the capacity of the queue
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Reserve additional capacity
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    /// Shrink the capacity to fit the current size
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
    }

    /// Enqueue an element (add to the back)
    pub fn enqueue(&mut self, value: T) {
        self.data.push_back(value);
    }

    /// Dequeue an element (remove from the front)
    pub fn dequeue(&mut self) -> Option<T> {
        self.data.pop_front()
    }

    /// Peek at the front element without removing it
    pub fn peek(&self) -> Option<&T> {
        self.data.front()
    }

    /// Peek at the front element mutably without removing it
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.data.front_mut()
    }

    /// Peek at the back element without removing it
    pub fn peek_back(&self) -> Option<&T> {
        self.data.back()
    }

    /// Peek at the back element mutably without removing it
    pub fn peek_back_mut(&mut self) -> Option<&mut T> {
        self.data.back_mut()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get an iterator over the elements
    pub fn iter(&self) -> VecDequeIter<T> {
        self.data.iter()
    }

    /// Get a mutable iterator over the elements
    pub fn iter_mut(&mut self) -> std::collections::vec_deque::IterMut<T> {
        self.data.iter_mut()
    }
}

/// Last-in-first-out data structure
#[derive(Debug, Clone)]
pub struct AFNSStack<T> {
    data: Vec<T>,
}

impl<T> AFNSStack<T> {
    /// Create a new empty stack
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Create a new stack with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the stack is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the capacity of the stack
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Reserve additional capacity
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    /// Shrink the capacity to fit the current size
    pub fn shrink_to_fit(&mut self) {
        self.data.shrink_to_fit();
    }

    /// Push an element onto the stack
    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    /// Pop an element from the stack
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    /// Peek at the top element without removing it
    pub fn peek(&self) -> Option<&T> {
        self.data.last()
    }

    /// Peek at the top element mutably without removing it
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        self.data.last_mut()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get an iterator over the elements (from bottom to top)
    pub fn iter(&self) -> std::slice::Iter<T> {
        self.data.iter()
    }

    /// Get a mutable iterator over the elements (from bottom to top)
    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.data.iter_mut()
    }
}

/// Doubly linked list
#[derive(Debug, Clone)]
pub struct AFNSLinkedList<T> {
    data: Box<StdLinkedList<T>>,
}

impl<T> AFNSLinkedList<T> {
    /// Create a new empty linked list
    pub fn new() -> Self {
        Self {
            data: Box::new(StdLinkedList::new()),
        }
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the linked list is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Push an element to the front
    pub fn push_front(&mut self, value: T) {
        self.data.push_front(value);
    }

    /// Push an element to the back
    pub fn push_back(&mut self, value: T) {
        self.data.push_back(value);
    }

    /// Pop an element from the front
    pub fn pop_front(&mut self) -> Option<T> {
        self.data.pop_front()
    }

    /// Pop an element from the back
    pub fn pop_back(&mut self) -> Option<T> {
        self.data.pop_back()
    }

    /// Peek at the front element without removing it
    pub fn front(&self) -> Option<&T> {
        self.data.front()
    }

    /// Peek at the front element mutably without removing it
    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.data.front_mut()
    }

    /// Peek at the back element without removing it
    pub fn back(&self) -> Option<&T> {
        self.data.back()
    }

    /// Peek at the back element mutably without removing it
    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.data.back_mut()
    }

    /// Get an iterator over the elements
    pub fn iter(&self) -> LinkedListIter<T> {
        self.data.iter()
    }

    /// Get a mutable iterator over the elements
    pub fn iter_mut(&mut self) -> std::collections::linked_list::IterMut<T> {
        self.data.iter_mut()
    }

    /// Split the list at the given index
    pub fn split_off(&mut self, at: usize) -> AFNSLinkedList<T> {
        Self {
            data: Box::new(self.data.split_off(at)),
        }
    }

    /// Append another list to this one
    pub fn append(&mut self, other: &mut AFNSLinkedList<T>) {
        self.data.append(&mut other.data);
    }
}

/// Circular buffer with fixed capacity
#[derive(Debug, Clone)]
pub struct AFNSRingBuffer<T> {
    data: Vec<Option<T>>,
    head: usize,
    tail: usize,
    len: usize,
    capacity: usize,
}

impl<T> AFNSRingBuffer<T> {
    /// Create a new ring buffer with specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![None; capacity],
            head: 0,
            tail: 0,
            len: 0,
            capacity,
        }
    }

    /// Get the capacity of the ring buffer
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get the number of elements
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the ring buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Check if the ring buffer is full
    pub fn is_full(&self) -> bool {
        self.len == self.capacity
    }

    /// Push an element to the back
    pub fn push_back(&mut self, value: T) -> Result<(), String> {
        if self.is_full() {
            return Err("Ring buffer is full".to_string());
        }

        self.data[self.tail] = Some(value);
        self.tail = (self.tail + 1) % self.capacity;
        self.len += 1;
        Ok(())
    }

    /// Push an element to the front
    pub fn push_front(&mut self, value: T) -> Result<(), String> {
        if self.is_full() {
            return Err("Ring buffer is full".to_string());
        }

        self.head = if self.head == 0 {
            self.capacity - 1
        } else {
            self.head - 1
        };
        self.data[self.head] = Some(value);
        self.len += 1;
        Ok(())
    }

    /// Pop an element from the back
    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        self.tail = if self.tail == 0 {
            self.capacity - 1
        } else {
            self.tail - 1
        };
        let value = self.data[self.tail].take();
        self.len -= 1;
        value
    }

    /// Pop an element from the front
    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let value = self.data[self.head].take();
        self.head = (self.head + 1) % self.capacity;
        self.len -= 1;
        value
    }

    /// Peek at the front element without removing it
    pub fn front(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        self.data[self.head].as_ref()
    }

    /// Peek at the back element without removing it
    pub fn back(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }
        let back_index = if self.tail == 0 {
            self.capacity - 1
        } else {
            self.tail - 1
        };
        self.data[back_index].as_ref()
    }

    /// Clear all elements
    pub fn clear(&mut self) {
        for i in 0..self.capacity {
            self.data[i] = None;
        }
        self.head = 0;
        self.tail = 0;
        self.len = 0;
    }

    /// Get an element at the specified index
    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len {
            return None;
        }
        let actual_index = (self.head + index) % self.capacity;
        self.data[actual_index].as_ref()
    }

    /// Get a mutable reference to an element at the specified index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        if index >= self.len {
            return None;
        }
        let actual_index = (self.head + index) % self.capacity;
        self.data[actual_index].as_mut()
    }
}

/// Alias for RingBuffer
pub type AFNSCircularBuffer<T> = AFNSRingBuffer<T>;

// Type aliases for common use cases
pub type Array<T> = AFNSArray<T>;
pub type Map<K, V> = AFNSMap<K, V>;
pub type Set<T> = AFNSSet<T>;
pub type Queue<T> = AFNSQueue<T>;
pub type Stack<T> = AFNSStack<T>;
pub type AFNSList<T> = AFNSLinkedList<T>;
pub type RingBuffer<T> = AFNSRingBuffer<T>;
pub type CircularBuffer<T> = AFNSCircularBuffer<T>;
