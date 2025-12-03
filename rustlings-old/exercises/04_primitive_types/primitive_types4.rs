fn main() {
    // You can optionally experiment here.
}

#[cfg(test)]
mod tests {
    #[test]
    fn slice_out_of_array() {
        let a = [1, 2, 3, 4, 5];

        // TODO: Get a slice called `nice_slice` out of the array `a` so that the test passes.
        // let nice_slice = ???
<<<<<<< HEAD
||||||| (empty tree)
=======
        let nice_slice = &a[1..4];
>>>>>>> 85b7b06 (initial commit)

        assert_eq!([2, 3, 4], nice_slice);
    }
}
