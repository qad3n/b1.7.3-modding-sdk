use crate::rt::call;
use crate::{IntoName, Result, expectRule, expectUnit};
use mod_protocol::Identifier;
use mod_protocol::abi::{GameRuleValue, HostCall};

/// needs api 1.32, a vanilla rule such as minecraft:keep_inventory or one a mod registered, a multiplayer client
/// only sees defaults
pub fn gameRule(rule: impl IntoName) -> Result<GameRuleValue>
{
    let rule: Identifier = rule.vanillaName()?;
    expectRule(call(HostCall::GameRule { rule }))
}

/// needs api 1.32, changes the rule for every dimension of the save
pub fn setGameRule(rule: impl IntoName, value: GameRuleValue) -> Result<()>
{
    let rule: Identifier = rule.vanillaName()?;
    expectUnit(call(HostCall::SetGameRule { rule, value }))
}
