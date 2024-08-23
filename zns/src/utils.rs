pub fn vec_equal<T: PartialEq>(vec1: &[T], vec2: &[T]) -> bool {
    if vec1.len() != vec2.len() {
        return false;
    }

    for (elem1, elem2) in vec1.iter().zip(vec2.iter()) {
        if elem1 != elem2 {
            return false;
        }
    }

    true
}
