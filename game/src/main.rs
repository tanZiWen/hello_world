use std::io;
use rand::Rng;
fn main() {
    println!("Guess the number game started!");

    loop{
        println!("Please type a number between 1 and 100:");

        let mut guess = String::new();

        io::stdin().read_line(&mut guess).expect("Failed to read line");
        
        println!("You guessed: {}", guess);

        let r: u32 = rand::rng().random_range(1..101);
        
        let guess_num = match guess.trim().parse::<u32>() {
            Ok(num) => num,
            Err(_) => {
                println!("Worng input, Please type a number between 1 and 100.");
                continue;
            }
        };

        if guess_num < 1 || guess_num > 100 {
            println!("Worng input, Please type a number between 1 and 100.");
            continue;
        }

        if r == guess_num {
            println!("Congratulations! You guessed correctly.");
            break;
        } else {
            println!("Sorry, the number was {}.", r);
        }
    }
}
