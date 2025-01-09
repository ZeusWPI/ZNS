use std::fmt::Display;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct LabelString(Vec<String>);

pub fn labels_equal(vec1: &LabelString, vec2: &LabelString) -> bool {
    if vec1.as_slice().len() != vec2.as_slice().len() {
        return false;
    }

    for (elem1, elem2) in vec1.as_slice().iter().zip(vec2.as_slice().iter()) {
        if elem1.to_lowercase() != elem2.to_lowercase() {
            return false;
        }
    }

    true
}

impl LabelString {
    pub fn from(string: &str) -> Self {
        LabelString(string.split('.').map(str::to_string).collect())
    }

    pub fn as_slice(&self) -> &[String] {
        self.0.as_slice()
    }

    pub fn to_vec(self) -> Vec<String> {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[cfg(feature = "test-utils")]
    pub fn prepend(&self, element: String) -> Self {
        let mut vec = self.0.clone();
        vec.insert(0, element);
        LabelString(vec)
    }
}

impl PartialEq for LabelString {
    fn eq(&self, other: &Self) -> bool {
        labels_equal(self, other)
    }
}

impl From<&[String]> for LabelString {
    fn from(value: &[String]) -> Self {
        LabelString(value.to_vec())
    }
}

impl From<Vec<String>> for LabelString {
    fn from(value: Vec<String>) -> Self {
        LabelString(value)
    }
}

impl Display for LabelString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_labels_equal() {
        assert!(labels_equal(
            &LabelString::from("one.two"),
            &LabelString::from("oNE.two")
        ));

        assert!(!labels_equal(
            &LabelString::from("onne.two"),
            &LabelString::from("oNEe.two")
        ));
    }
}
