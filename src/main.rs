use crate::types::{Primitive, Type};

#[allow(dead_code)]
mod types;

fn main() {
    println!("Hello, world!");
    let boolean_instance = Primitive::Boolean;

    // Example usage
    let is_subtype = boolean_instance.is_assignable_to(&boolean_instance);
    println!("Is subtype: {}", is_subtype); // Output should be true
}
