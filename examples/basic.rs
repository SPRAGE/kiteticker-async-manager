//! Basic example demonstrating the template functionality

use my_rust_project::{greet, add};

fn main() {
    println!("=== Basic Example ===");
    
    // Test the greet function
    let greeting = greet("Template User");
    println!("{}", greeting);
    
    // Test the add function
    let result = add(5, 3);
    println!("5 + 3 = {}", result);
    
    println!("Basic example completed successfully!");
}
