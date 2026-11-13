use crate::rt::call;
use crate::{Error, Result, dispatch, expectText, expectUnit};
use mod_protocol::abi::{ConfigKind, ConfigOption, ConfigScope, MetaRequest};
use std::fmt::Display;
use std::ops::RangeInclusive;
use std::rc::Rc;

/// one option for Config::options, scope defaults to common
pub struct Setting
{
    m_option: ConfigOption,
}

impl Setting
{
    fn new(key: &str, kind: ConfigKind) -> Setting
    {
        Setting
        {
            m_option: ConfigOption
            {
                key: key.to_owned(),
                label: String::new(),
                comment: String::new(),
                scope: ConfigScope::Common,
                kind,
                restart: false,
            },
        }
    }

    pub fn bool(key: &str, default: bool) -> Setting { Setting::new(key, ConfigKind::Bool { default }) }

    pub fn int(key: &str, default: i64, range: RangeInclusive<i64>) -> Setting
    {
        Setting::new(key, ConfigKind::Int { default, min: *range.start(), max: *range.end() })
    }

    pub fn float(key: &str, default: f64, range: RangeInclusive<f64>) -> Setting
    {
        Setting::new(key, ConfigKind::Float { default, min: *range.start(), max: *range.end() })
    }

    pub fn text(key: &str, default: &str) -> Setting
    {
        Setting::new(key, ConfigKind::Text { default: default.to_owned() })
    }

    pub fn choice(key: &str, default: &str, choices: &[&str]) -> Setting
    {
        let choices: Vec<String> = choices.iter().map(|choice| (*choice).to_owned()).collect();
        Setting::new(key, ConfigKind::Choice { default: default.to_owned(), choices })
    }

    pub fn label(mut self, label: &str) -> Self
    {
        self.m_option.label = label.to_owned();
        self
    }

    pub fn comment(mut self, comment: &str) -> Self
    {
        self.m_option.comment = comment.to_owned();
        self
    }

    /// only this machine reads it, never synced
    pub fn client(mut self) -> Self
    {
        self.m_option.scope = ConfigScope::Client;
        self
    }

    /// the server value wins on every client while connected
    pub fn server(mut self) -> Self
    {
        self.m_option.scope = ConfigScope::Server;
        self
    }

    pub fn restart(mut self) -> Self
    {
        self.m_option.restart = true;
        self
    }
}

pub struct Config;

impl Config
{
    /// values live in config/<modid>.toml and the player edits them in the mod menu
    pub fn options(&self, settings: impl IntoIterator<Item = Setting>) -> Result<()>
    {
        let options: Vec<ConfigOption> = settings.into_iter().map(|setting| setting.m_option).collect();
        expectUnit(call(MetaRequest::DefineConfigOptions { options }))
    }

    pub fn get(&self, key: &str) -> String { configValue(key) }

    pub fn getBool(&self, key: &str) -> bool { configValue(key) == "true" }

    pub fn getInt(&self, key: &str) -> i64 { configValue(key).parse().unwrap_or_default() }

    pub fn getFloat(&self, key: &str) -> f64 { configValue(key).parse().unwrap_or_default() }

    pub fn set(&self, key: &str, value: impl Display) -> Result<()>
    {
        expectUnit(call(MetaRequest::SetConfigValue { key: key.to_owned(), value: value.to_string() }))
            .map_err(|error| Error(format!("config {key}: {}", error.0)))
    }

    /// runs with the key after the player changes an option in the mod menu
    pub fn onChange(&self, handler: impl Fn(&str) + 'static) { dispatch::addConfigListener(Rc::new(handler)); }
}

pub fn configValue(key: &str) -> String
{
    expectText(call(MetaRequest::ConfigValue { key: key.to_owned() })).unwrap_or_default()
}
