fn main() {
let result: Result<i32, &str> = Err("Something went wrong");
match result {
Ok(value) => println!("Success: {}", value),
Err(e) => println!("Error: {}", e),
}
}
