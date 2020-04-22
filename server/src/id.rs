/// An uppercase identifier.
pub struct Id(String);

impl Id {
    pub fn as_str(&self) -> &str { &self.0 }
}

impl From<Id> for String {
    fn from(Id(s): Id) -> Self { s }
}

impl std::str::FromStr for Id {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_uppercase()))
    }
}
