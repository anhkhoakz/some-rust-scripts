<<<<<<< HEAD
// Lifetimes are also needed when structs hold references.

struct Book<'a> {
    //     ^^^^ added a lifetime annotation
    author: &'a str,
    //       ^^
    title: &'a str,
    //      ^^
}

fn main() {
    let book = Book {
        author: "George Orwell",
        title: "1984",
    };

    println!("{} by {}", book.title, book.author);
||||||| (empty tree)
=======
fn main() {
    // DON'T EDIT THIS SOLUTION FILE!
    // It will be automatically filled after you finish the exercise.
>>>>>>> 85b7b06 (initial commit)
}
