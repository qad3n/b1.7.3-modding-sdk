use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

pub const MINECRAFT: &str = "minecraft";

mod limits
{
    pub const MOD_ID_MIN: usize = 2;
    pub const NAMESPACE_MAX: usize = 64;
    pub const PATH_MAX: usize = 256;
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Identifier
{
    m_namespace: String,
    m_path: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum IdentifierError
{
    MissingSeparator(String),
    BadNamespace(String),
    BadPath(String),
}

impl fmt::Display for IdentifierError
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            IdentifierError::MissingSeparator(text) => write!(formatter, "identifier '{text}' has no namespace"),
            IdentifierError::BadNamespace(text) =>
            {
                write!(formatter, "namespace '{text}' must match [a-z0-9_]{{1,64}}")
            }
            IdentifierError::BadPath(text) => write!(formatter, "path '{text}' must match [a-z0-9_./-]{{1,256}}"),
        }
    }
}

impl std::error::Error for IdentifierError {}

// shared namespaces such as c for common tags may be one letter, mod ids need two
pub fn isValidNamespace(namespace: &str) -> bool
{
    (1..=limits::NAMESPACE_MAX).contains(&namespace.len())
        && namespace.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

pub fn isValidModId(id: &str) -> bool { id.len() >= limits::MOD_ID_MIN && isValidNamespace(id) }

fn isValidPath(path: &str) -> bool
{
    (1..=limits::PATH_MAX).contains(&path.len())
        && path
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'/' | b'-'))
}

impl Identifier
{
    pub fn new(namespace: &str, path: &str) -> Result<Identifier, IdentifierError>
    {
        if !isValidNamespace(namespace)
        {
            return Err(IdentifierError::BadNamespace(namespace.to_owned()));
        }
        if !isValidPath(path)
        {
            return Err(IdentifierError::BadPath(path.to_owned()));
        }
        Ok(Identifier { m_namespace: namespace.to_owned(), m_path: path.to_owned() })
    }

    pub fn minecraft(path: &str) -> Identifier
    {
        Identifier::new(MINECRAFT, path).expect("vanilla identifiers are written valid")
    }

    pub fn parse(text: &str) -> Result<Identifier, IdentifierError>
    {
        match text.split_once(':')
        {
            Some((namespace, path)) => Identifier::new(namespace, path),
            None => Err(IdentifierError::MissingSeparator(text.to_owned())),
        }
    }

    // a bare path resolves into the given namespace, as mods write their own content
    pub fn parseIn(text: &str, namespace: &str) -> Result<Identifier, IdentifierError>
    {
        if text.contains(':')
        {
            Identifier::parse(text)
        }
        else
        {
            Identifier::new(namespace, text)
        }
    }

    pub fn namespace(&self) -> &str { &self.m_namespace }

    pub fn path(&self) -> &str { &self.m_path }

    pub fn isVanilla(&self) -> bool { self.m_namespace == MINECRAFT }
}

impl fmt::Display for Identifier
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        write!(formatter, "{}:{}", self.m_namespace, self.m_path)
    }
}

impl std::str::FromStr for Identifier
{
    type Err = IdentifierError;

    fn from_str(text: &str) -> Result<Identifier, IdentifierError> { Identifier::parse(text) }
}

impl Serialize for Identifier
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> { serializer.collect_str(self) }
}

impl<'de> Deserialize<'de> for Identifier
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Identifier, D::Error>
    {
        let text: String = String::deserialize(deserializer)?;
        Identifier::parse(&text).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn parsesAndValidates()
    {
        let id: Identifier = Identifier::parse("ruby:block/ruby_ore").expect("valid");
        assert_eq!(id.namespace(), "ruby");
        assert_eq!(id.path(), "block/ruby_ore");
        assert!(Identifier::parse("Ruby:x").is_err());
        assert!(Identifier::parse("ruby").is_err());
        assert_eq!(Identifier::parseIn("gem", "ruby").expect("valid").to_string(), "ruby:gem");
    }

    #[test]
    fn roundTripsThroughPostcard()
    {
        let id: Identifier = Identifier::minecraft("stone");
        let bytes: Vec<u8> = postcard::to_allocvec(&id).expect("encodes");
        let back: Identifier = postcard::from_bytes(&bytes).expect("decodes");
        assert_eq!(id, back);
    }
}
