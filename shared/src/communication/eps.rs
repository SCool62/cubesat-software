use core::{num::ParseIntError, str::Utf8Error};

use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EpsCommand {
    EnablePowerRail(u8),
    DisablePowerRail(u8),
    StateOfHealthReq,
    GetBatteryVoltage(u8),
    GetPowerRailState(u8),
}

impl EpsCommand {
    pub fn from_bytes(bytes: &[u8]) -> Result<EpsCommand, CommandParseError> {
        let mut words = bytes.split(|b| b == b";".iter().next().unwrap());
        match words.next() {
            Some(word) => {
                match word {
                    b"pwe" => {
                        let arg = words.next().ok_or(CommandParseError::IncompleteArgs)?;
                        let rail_num = core::str::from_utf8(arg)?.parse::<u8>()?;
                        Ok(EpsCommand::EnablePowerRail(rail_num))
                    },
                    b"pwd" => {
                        let arg = words.next().ok_or(CommandParseError::IncompleteArgs)?;
                        let rail_num = core::str::from_utf8(arg)?.parse::<u8>()?;
                        Ok(EpsCommand::DisablePowerRail(rail_num))
                    },
                    b"soh" => return Ok(Self::StateOfHealthReq),
                    b"gbv" => {
                        let arg = words.next().ok_or(CommandParseError::IncompleteArgs)?;
                        let rail_num = core::str::from_utf8(arg)?.parse::<u8>()?;
                        Ok(EpsCommand::GetBatteryVoltage(rail_num))
                    },
                    b"gprs" => {
                        let arg = words.next().ok_or(CommandParseError::IncompleteArgs)?;
                        let rail_num = core::str::from_utf8(arg)?.parse::<u8>()?;
                        Ok(EpsCommand::GetPowerRailState(rail_num))
                    },
                    _ => Err(CommandParseError::CommandNotFound)
                }
            },
            None => Err(CommandParseError::EmptyMessage)
        }
    }
}

#[derive(Error, Debug)]
pub enum CommandParseError {
    #[error("Command not Found")]
    CommandNotFound,
    #[error("Empty message buffer")]
    EmptyMessage,
    #[error("Incomplete args")]
    IncompleteArgs,
    #[error("ParseIntError {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("Utf8Error {0}")]
    Utf8Error(#[from] Utf8Error)
}