//! Async example demonstrating async functionality
//! 
//! Run with: cargo run --example async --features async

#[cfg(feature = "async")]
use my_rust_project::greet_async;

#[cfg(feature = "async")]
#[tokio::main]
async fn main() {
    println!("=== Async Example ===");
    
    let greeting = greet_async("Async World").await;
    println!("{}", greeting);
    
    // Demonstrate multiple concurrent greetings
    let futures = vec![
        greet_async("Alice"),
        greet_async("Bob"),
        greet_async("Charlie"),
    ];
    
    let greetings = futures::future::join_all(futures).await;
    
    for greeting in greetings {
        println!("{}", greeting);
    }
    
    println!("Async example completed successfully!");
}

#[cfg(not(feature = "async"))]
fn main() {
    println!("This example requires the 'async' feature.");
    println!("Run with: cargo run --example async --features async");
}
