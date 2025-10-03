//! Structs library for AFNS
//! 
//! This module provides fundamental data structures including:
//! - Tuple: Fixed-size heterogeneous collection
//! - Option: Represents optional values (Some or None)
//! - Result: Represents success (Ok) or failure (Err)

use std::fmt;
use std::ops::{Index, IndexMut};

/// Fixed-size heterogeneous collection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AFNSTuple<T> {
    data: Vec<T>,
}

impl<T> AFNSTuple<T> {
    /// Create a new tuple with the given elements
    pub fn new(elements: Vec<T>) -> Self {
        Self { data: elements }
    }

    /// Get the length of the tuple
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the tuple is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
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

impl<T> Index<usize> for AFNSTuple<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T> IndexMut<usize> for AFNSTuple<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<T> fmt::Display for AFNSTuple<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        for (i, item) in self.data.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", item)?;
        }
        write!(f, ")")
    }
}

/// Represents optional values (Some or None)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AFNSOption<T> {
    Some(T),
    None,
}

impl<T> AFNSOption<T> {
    /// Create a Some value
    pub fn some(value: T) -> Self {
        Self::Some(value)
    }

    /// Create a None value
    pub fn none() -> Self {
        Self::None
    }

    /// Check if the option is Some
    pub fn is_some(&self) -> bool {
        matches!(self, Self::Some(_))
    }

    /// Check if the option is None
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Unwrap the value, panicking if None
    pub fn unwrap(self) -> T {
        match self {
            Self::Some(value) => value,
            Self::None => panic!("Called unwrap on a None value"),
        }
    }

    /// Unwrap the value with a default if None
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Some(value) => value,
            Self::None => default,
        }
    }

    /// Unwrap the value with a computed default if None
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            Self::Some(value) => value,
            Self::None => f(),
        }
    }

    /// Map the value if Some
    pub fn map<U, F>(self, f: F) -> AFNSOption<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Some(value) => AFNSOption::Some(f(value)),
            Self::None => AFNSOption::None,
        }
    }

    /// Map the value if Some, otherwise return the default
    pub fn map_or<U, F>(self, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Some(value) => f(value),
            Self::None => default,
        }
    }

    /// Map the value if Some, otherwise compute the default
    pub fn map_or_else<U, F, D>(self, default: D, f: F) -> U
    where
        F: FnOnce(T) -> U,
        D: FnOnce() -> U,
    {
        match self {
            Self::Some(value) => f(value),
            Self::None => default(),
        }
    }

    /// Apply a function to the value if Some
    pub fn and_then<U, F>(self, f: F) -> AFNSOption<U>
    where
        F: FnOnce(T) -> AFNSOption<U>,
    {
        match self {
            Self::Some(value) => f(value),
            Self::None => AFNSOption::None,
        }
    }

    /// Return the option if Some, otherwise return the other option
    pub fn or(self, optb: AFNSOption<T>) -> AFNSOption<T> {
        match self {
            Self::Some(value) => Self::Some(value),
            Self::None => optb,
        }
    }

    /// Return the option if Some, otherwise compute the other option
    pub fn or_else<F>(self, f: F) -> AFNSOption<T>
    where
        F: FnOnce() -> AFNSOption<T>,
    {
        match self {
            Self::Some(value) => Self::Some(value),
            Self::None => f(),
        }
    }

    /// Get the value if Some, otherwise return None
    pub fn as_ref(&self) -> AFNSOption<&T> {
        match self {
            Self::Some(value) => AFNSOption::Some(value),
            Self::None => AFNSOption::None,
        }
    }

    /// Get a mutable reference to the value if Some, otherwise return None
    pub fn as_mut(&mut self) -> AFNSOption<&mut T> {
        match self {
            Self::Some(value) => AFNSOption::Some(value),
            Self::None => AFNSOption::None,
        }
    }

    /// Take the value out of the option, leaving None in its place
    pub fn take(&mut self) -> AFNSOption<T> {
        std::mem::replace(self, AFNSOption::None)
    }

    /// Replace the value in the option with the given value
    pub fn replace(&mut self, value: T) -> AFNSOption<T> {
        std::mem::replace(self, AFNSOption::Some(value))
    }

    /// Filter the option based on a predicate
    pub fn filter<P>(self, predicate: P) -> AFNSOption<T>
    where
        P: FnOnce(&T) -> bool,
    {
        match self {
            Self::Some(value) if predicate(&value) => Self::Some(value),
            _ => Self::None,
        }
    }

    /// Zip two options into a tuple option
    pub fn zip<U>(self, other: AFNSOption<U>) -> AFNSOption<(T, U)> {
        match (self, other) {
            (Self::Some(a), AFNSOption::Some(b)) => AFNSOption::Some((a, b)),
            _ => AFNSOption::None,
        }
    }

    /// Unzip a tuple option into two options
    pub fn unzip<U, V>(self) -> (AFNSOption<U>, AFNSOption<V>)
    where
        T: Into<(U, V)>,
    {
        match self {
            Self::Some(value) => {
                let (a, b) = value.into();
                (AFNSOption::Some(a), AFNSOption::Some(b))
            }
            Self::None => (AFNSOption::None, AFNSOption::None),
        }
    }
}

impl<T> fmt::Display for AFNSOption<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Some(value) => write!(f, "Some({})", value),
            Self::None => write!(f, "None"),
        }
    }
}

/// Represents success (Ok) or failure (Err)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AFNSResult<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E> AFNSResult<T, E> {
    /// Create an Ok result
    pub fn ok(value: T) -> Self {
        Self::Ok(value)
    }

    /// Create an Err result
    pub fn err(error: E) -> Self {
        Self::Err(error)
    }

    /// Check if the result is Ok
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Check if the result is Err
    pub fn is_err(&self) -> bool {
        matches!(self, Self::Err(_))
    }

    /// Unwrap the Ok value, panicking if Err
    pub fn unwrap(self) -> T {
        match self {
            Self::Ok(value) => value,
            Self::Err(_) => panic!("Called unwrap on an Err value"),
        }
    }

    /// Unwrap the Ok value, panicking with a custom message if Err
    pub fn expect(self, msg: &str) -> T {
        match self {
            Self::Ok(value) => value,
            Self::Err(_) => panic!("{}", msg),
        }
    }

    /// Unwrap the Ok value, or return the default if Err
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Self::Ok(value) => value,
            Self::Err(_) => default,
        }
    }

    /// Unwrap the Ok value, or compute the default if Err
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce(E) -> T,
    {
        match self {
            Self::Ok(value) => value,
            Self::Err(error) => f(error),
        }
    }

    /// Map the Ok value
    pub fn map<U, F>(self, f: F) -> AFNSResult<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Ok(value) => AFNSResult::Ok(f(value)),
            Self::Err(error) => AFNSResult::Err(error),
        }
    }

    /// Map the Err value
    pub fn map_err<F, O>(self, f: F) -> AFNSResult<T, O>
    where
        F: FnOnce(E) -> O,
    {
        match self {
            Self::Ok(value) => AFNSResult::Ok(value),
            Self::Err(error) => AFNSResult::Err(f(error)),
        }
    }

    /// Map the Ok value, or return the default if Err
    pub fn map_or<U, F>(self, default: U, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Ok(value) => f(value),
            Self::Err(_) => default,
        }
    }

    /// Map the Ok value, or compute the default if Err
    pub fn map_or_else<U, F, D>(self, default: D, f: F) -> U
    where
        F: FnOnce(T) -> U,
        D: FnOnce(E) -> U,
    {
        match self {
            Self::Ok(value) => f(value),
            Self::Err(error) => default(error),
        }
    }

    /// Apply a function to the Ok value
    pub fn and_then<U, F>(self, f: F) -> AFNSResult<U, E>
    where
        F: FnOnce(T) -> AFNSResult<U, E>,
    {
        match self {
            Self::Ok(value) => f(value),
            Self::Err(error) => AFNSResult::Err(error),
        }
    }

    /// Return the result if Ok, otherwise return the other result
    pub fn or<F>(self, res: AFNSResult<T, F>) -> AFNSResult<T, F> {
        match self {
            Self::Ok(value) => AFNSResult::Ok(value),
            Self::Err(_) => res,
        }
    }

    /// Return the result if Ok, otherwise compute the other result
    pub fn or_else<F, O>(self, f: F) -> AFNSResult<T, O>
    where
        F: FnOnce(E) -> AFNSResult<T, O>,
    {
        match self {
            Self::Ok(value) => AFNSResult::Ok(value),
            Self::Err(error) => f(error),
        }
    }

    /// Get the Ok value if Ok, otherwise return None
    pub fn ok(self) -> AFNSOption<T> {
        match self {
            Self::Ok(value) => AFNSOption::Some(value),
            Self::Err(_) => AFNSOption::None,
        }
    }

    /// Get the Err value if Err, otherwise return None
    pub fn err(self) -> AFNSOption<E> {
        match self {
            Self::Ok(_) => AFNSOption::None,
            Self::Err(error) => AFNSOption::Some(error),
        }
    }

    /// Get a reference to the Ok value if Ok, otherwise return None
    pub fn as_ref(&self) -> AFNSResult<&T, &E> {
        match self {
            Self::Ok(value) => AFNSResult::Ok(value),
            Self::Err(error) => AFNSResult::Err(error),
        }
    }

    /// Get a mutable reference to the Ok value if Ok, otherwise return None
    pub fn as_mut(&mut self) -> AFNSResult<&mut T, &mut E> {
        match self {
            Self::Ok(value) => AFNSResult::Ok(value),
            Self::Err(error) => AFNSResult::Err(error),
        }
    }

    /// Transpose a Result of an Option into an Option of a Result
    pub fn transpose(self) -> AFNSOption<AFNSResult<T, E>>
    where
        T: Into<AFNSOption<T>>,
    {
        match self {
            Self::Ok(value) => {
                let option: AFNSOption<T> = value.into();
                match option {
                    AFNSOption::Some(v) => AFNSOption::Some(AFNSResult::Ok(v)),
                    AFNSOption::None => AFNSOption::None,
                }
            }
            Self::Err(error) => AFNSOption::Some(AFNSResult::Err(error)),
        }
    }

    /// Flatten a nested Result
    pub fn flatten<U>(self) -> AFNSResult<U, E>
    where
        T: Into<AFNSResult<U, E>>,
    {
        match self {
            Self::Ok(value) => value.into(),
            Self::Err(error) => AFNSResult::Err(error),
        }
    }
}

impl<T, E> fmt::Display for AFNSResult<T, E>
where
    T: fmt::Display,
    E: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ok(value) => write!(f, "Ok({})", value),
            Self::Err(error) => write!(f, "Err({})", error),
        }
    }
}

// Type aliases for common use cases
pub type Tuple<T> = AFNSTuple<T>;
pub type Option<T> = AFNSOption<T>;
pub type Result<T, E> = AFNSResult<T, E>;
