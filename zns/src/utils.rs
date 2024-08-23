use crate::structs::LabelString;

pub fn labels_equal(vec1: &LabelString, vec2: &LabelString) -> bool {
    if vec1.len() != vec2.len() {
        return false;
    }

    for (elem1, elem2) in vec1.iter().zip(vec2.iter()) {
        if elem1.to_lowercase() != elem2.to_lowercase() {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_labels_equal() {
        assert!(labels_equal(
            &vec![String::from("one"), String::from("two")],
            &vec![String::from("oNE"), String::from("two")]
        ));

        assert!(!labels_equal(
            &vec![String::from("one"), String::from("two")],
            &vec![String::from("oNEe"), String::from("two")]
        ));
    }
}
