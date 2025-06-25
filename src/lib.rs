//! # My Rust Project
//!
//! A Rust project created from the enhanced template with major-version-only branching.
//!
//! ## Quick Start
//!
//! ```rust
//! use my_rust_project::greet;
//!
//! fn main() {
//!     println!("{}", greet("World"));
//! }
//! ```
//!
//! ## Features
//!
//! - `serde`: Enable serialization support
//! - `async`: Enable async/await support

/// Greets someone with a friendly message.
///
/// # Examples
///
/// ```
/// use my_rust_project::greet;
///
/// assert_eq!(greet("Rust"), "Hello, Rust!");
/// ```
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

/// Adds two numbers together.
///
/// # Examples
///
/// ```
/// use my_rust_project::add;
///
/// assert_eq!(add(2, 3), 5);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(feature = "async")]
/// Async version of greet function.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "async")]
/// # async fn example() {
/// use my_rust_project::greet_async;
///
/// let greeting = greet_async("Async Rust").await;
/// assert_eq!(greeting, "Hello, Async Rust!");
/// # }
/// ```
pub async fn greet_async(name: &str) -> String {
    // Simulate some async work
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    format!("Hello, {}!", name)
}

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "serde")]
/// A person struct that can be serialized/deserialized.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub age: u32,
}

#[cfg(feature = "serde")]
impl Person {
    /// Creates a new person.
    pub fn new(name: String, age: u32) -> Self {
        Self { name, age }
    }

    /// Greets the person.
    pub fn greet(&self) -> String {
        format!("Hello, {}! You are {} years old.", self.name, self.age)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet() {
        assert_eq!(greet("Test"), "Hello, Test!");
    }

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_greet_async() {
        let result = greet_async("Async Test").await;
        assert_eq!(result, "Hello, Async Test!");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_person() {
        let person = Person::new("Alice".to_string(), 30);
        assert_eq!(person.greet(), "Hello, Alice! You are 30 years old.");
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_person_serialization() {
        let person = Person::new("Bob".to_string(), 25);
        let json = serde_json::to_string(&person).unwrap();
        let deserialized: Person = serde_json::from_str(&json).unwrap();
        assert_eq!(person, deserialized);
    }
}
