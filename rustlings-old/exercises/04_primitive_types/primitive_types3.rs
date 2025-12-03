fn main() {
    // TODO: Create an array called `a` with at least 100 elements in it.
    // let a = ???
<<<<<<< HEAD
||||||| (empty tree)
=======
    let a = [1; 200];
>>>>>>> 85b7b06 (initial commit)

    if a.len() >= 100 {
        println!("Wow, that's a big array!");
    } else {
        println!("Meh, I eat arrays like that for breakfast.");
        panic!("Array not big enough, more elements needed");
    }
}
