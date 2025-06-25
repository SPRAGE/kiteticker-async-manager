//! Serde example demonstrating serialization features
//!
//! Run with: cargo run --example serde --features serde

#[cfg(feature = "serde")]
use my_rust_project::Person;

#[cfg(feature = "serde")]
fn main() {
    println!("=== Serde Example ===");
    
    // Create a person
    let person = Person::new("Alice".to_string(), 30);
    println!("{}", person.greet());
    
    // Serialize to JSON
    let json = serde_json::to_string_pretty(&person).unwrap();
    println!("Serialized to JSON:\n{}", json);
    
    // Deserialize from JSON
    let deserialized: Person = serde_json::from_str(&json).unwrap();
    println!("Deserialized: {}", deserialized.greet());
    
    // Verify they're equal
    assert_eq!(person, deserialized);
    println!("Serialization round-trip successful!");
    
    println!("Serde example completed successfully!");
}

#[cfg(not(feature = "serde"))]
fn main() {
    println!("This example requires the 'serde' feature.");
    println!("Run with: cargo run --example serde --features serde");
}
