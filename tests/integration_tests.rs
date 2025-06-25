//! Integration tests for the template

use my_rust_project::{greet, add};

#[test]
fn test_greet_integration() {
    let result = greet("Integration Test");
    assert_eq!(result, "Hello, Integration Test!");
}

#[test]
fn test_add_integration() {
    assert_eq!(add(10, 20), 30);
    assert_eq!(add(-5, 5), 0);
    assert_eq!(add(0, 0), 0);
}

#[cfg(feature = "async")]
#[tokio::test]
async fn test_async_greet_integration() {
    use my_rust_project::greet_async;
    
    let result = greet_async("Async Integration").await;
    assert_eq!(result, "Hello, Async Integration!");
}

#[cfg(feature = "serde")]
#[test]
fn test_serde_integration() {
    use my_rust_project::Person;
    
    let person = Person::new("Test User".to_string(), 25);
    
    // Test serialization round-trip
    let json = serde_json::to_string(&person).unwrap();
    let deserialized: Person = serde_json::from_str(&json).unwrap();
    
    assert_eq!(person, deserialized);
    assert_eq!(person.greet(), "Hello, Test User! You are 25 years old.");
}
