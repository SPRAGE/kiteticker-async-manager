fn main() {
    println!("{}", my_rust_project::greet("World"));
    println!("2 + 3 = {}", my_rust_project::add(2, 3));

    #[cfg(feature = "serde")]
    {
        let person = my_rust_project::Person::new("Alice".to_string(), 30);
        println!("{}", person.greet());
    }
}
