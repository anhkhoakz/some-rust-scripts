fn array_and_vec() -> ([i32; 4], Vec<i32>) {
    let a = [10, 20, 30, 40]; // Array

    // TODO: Create a vector called `v` which contains the exact same elements as in the array `a`.
    // Use the vector macro.
    // let v = ???;

<<<<<<< HEAD
    (a, v)
||||||| (empty tree)
=======
    let v = a.to_vec();
    (a, v)

    // let b = Vec::new();
    // b.push(10);
    // b.push(20);
    // b.push(30);
    // b.push(40);
    //
    // let c = vec![10, 20, 30, 40]
>>>>>>> 85b7b06 (initial commit)
}

fn main() {
    // You can optionally experiment here.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_and_vec_similarity() {
        let (a, v) = array_and_vec();
        assert_eq!(a, *v);
    }
}
