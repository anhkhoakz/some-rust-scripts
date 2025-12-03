fn main() {
    // You can optionally experiment here.
}

#[cfg(test)]
mod tests {
    // TODO: Fix the compiler errors only by reordering the lines in the test.
    // Don't add, change or remove any line.
    #[test]
    fn move_semantics4() {
        let mut x = Vec::new();
        let y = &mut x;
<<<<<<< HEAD
        let z = &mut x;
        y.push(42);
||||||| (empty tree)
=======
        y.push(42);
        let z = &mut x;
>>>>>>> 85b7b06 (initial commit)
        z.push(13);
        assert_eq!(x, [42, 13]);
    }
}
