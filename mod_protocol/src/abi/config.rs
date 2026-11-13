use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConfigScope
{
    Client,
    Common,
    Server,
}

/// a typed option, key is name or category.name and a category becomes a table in config/<modid>.toml
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct ConfigOption
{
    pub key: String,
    /// shown in the mod menu, empty uses the name part of the key
    pub label: String,
    pub comment: String,
    pub scope: ConfigScope,
    pub kind: ConfigKind,
    /// the mod reads the value once, so a change shows after a restart
    pub restart: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ConfigKind
{
    Bool
    {
        default: bool
    },

    Int
    {
        default: i64, min: i64, max: i64
    },

    Float
    {
        default: f64, min: f64, max: f64
    },

    Text
    {
        default: String
    },

    Choice
    {
        default: String, choices: Vec<String>
    },
}

impl ConfigKind
{
    pub fn defaultText(&self) -> String
    {
        match self
        {
            ConfigKind::Bool { default } => default.to_string(),
            ConfigKind::Int { default, .. } => default.to_string(),
            ConfigKind::Float { default, .. } => default.to_string(),
            ConfigKind::Text { default } | ConfigKind::Choice { default, .. } => default.clone(),
        }
    }

    /// the canonical text of a value, or why it does not fit this option
    pub fn check(&self, text: &str) -> Result<String, String>
    {
        let trimmed: &str = text.trim();

        match self
        {
            ConfigKind::Bool { .. } => match trimmed.to_ascii_lowercase().as_str()
            {
                "true" => Ok("true".to_owned()),
                "false" => Ok("false".to_owned()),
                _ => Err(format!("'{trimmed}' is not true or false")),
            },

            ConfigKind::Int { min, max, .. } =>
            {
                let value: i64 = trimmed.parse().map_err(|_| format!("'{trimmed}' is not a whole number"))?;
                inRange(value, *min, *max).map(|value| value.to_string())
            }

            ConfigKind::Float { min, max, .. } =>
            {
                let value: f64 = trimmed
                    .parse()
                    .ok()
                    .filter(|value: &f64| value.is_finite())
                    .ok_or_else(|| format!("'{trimmed}' is not a number"))?;
                inRange(value, *min, *max).map(|value| value.to_string())
            }

            ConfigKind::Text { .. } => Ok(text.to_owned()),
            ConfigKind::Choice { choices, .. } => choices
                .iter()
                .find(|choice| choice.as_str() == trimmed)
                .cloned()
                .ok_or_else(|| format!("'{trimmed}' is not one of {}", choices.join(", "))),
        }
    }
}

fn inRange<T: PartialOrd + std::fmt::Display>(value: T, min: T, max: T) -> Result<T, String>
{
    if value < min || value > max
    {
        return Err(format!("{value} is outside {min} to {max}"));
    }

    Ok(value)
}
