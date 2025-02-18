use std::collections::HashMap;
fn main() {
   let teams = vec![String::from("blue"), String::from("red")];
   let initial_score = vec![10, 50];

   let mut scores: HashMap<String, i32> = teams.into_iter().zip(initial_score.into_iter()).collect();

   println!("{:?}", scores);  // Output: {"blue": 10, "red": 50}

   let field_name = String::from("Favorite color");

   let updated_score = scores.entry(field_name).or_insert(0);

   *updated_score += 100;

   println!("{:?}", scores);  // Output: {"blue": 10, "red": 50, "Favorite color": 10}

   scores.insert(String::from("You"), 1000);

   println!("{:?}", scores); // Output: {"blue": 10, "red": 50, "Favorite color": 10, "You": 1000}

   scores.insert(String::from("blue"), 10000);

   println!("{:?}", scores); // Output: {"blue": 10000, "red": 50, "Favorite color": 10, "You": 1000}

}