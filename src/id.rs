#[derive(Debug, Clone, std::cmp::PartialEq, std::cmp::Eq, std::hash::Hash)]
pub struct Id(String);
impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl From<String> for Id {
    fn from(s: String) -> Self {
        Id(s)
    }
}
impl From<&str> for Id {
    fn from(s: &str) -> Self {
        Id(s.to_string())
    }
}