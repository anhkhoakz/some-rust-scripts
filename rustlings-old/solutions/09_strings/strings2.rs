<<<<<<< HEAD
fn is_a_color_word(attempt: &str) -> bool {
    attempt == "green" || attempt == "blue" || attempt == "red"
}

fn main() {
    let word = String::from("green");

    if is_a_color_word(&word) {
        //             ^ added to have `&String` which is automatically
        //               coerced to `&str` by the compiler.
        println!("That is a color word I know!");
    } else {
        println!("That is not a color word I know.");
    }
||||||| (empty tree)
=======
fn main() {
    // DON'T EDIT THIS SOLUTION FILE!
    // It will be automatically filled after you finish the exercise.
>>>>>>> 85b7b06 (initial commit)
}
