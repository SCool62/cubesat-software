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
            Some(word) => match word {
                b"pwe" => {
                    let arg = words.next().ok_or(CommandParseError::IncompleteArgs)?;
                    let rail_num = core::str::from_utf8(arg)?.parse::<u8>()?;
                    Ok(EpsCommand::EnablePowerRail(rail_num))
                }
                b"pwd" => {
                    let arg = words.next().ok_or(CommandParseError::IncompleteArgs)?;
                    let rail_num = core::str::from_utf8(arg)?.parse::<u8>()?;
                    Ok(EpsCommand::DisablePowerRail(rail_num))
                }
                b"soh" => Ok(Self::StateOfHealthReq),
                b"gbv" => {
                    let arg = words.next().ok_or(CommandParseError::IncompleteArgs)?;
                    let rail_num = core::str::from_utf8(arg)?.parse::<u8>()?;
                    Ok(EpsCommand::GetBatteryVoltage(rail_num))
                }
                b"gprs" => {
                    let arg = words.next().ok_or(CommandParseError::IncompleteArgs)?;
                    let rail_num = core::str::from_utf8(arg)?.parse::<u8>()?;
                    Ok(EpsCommand::GetPowerRailState(rail_num))
                }
                _ => Err(CommandParseError::UnknownCommand),
            },
            None => Err(CommandParseError::EmptyMessage),
        }
    }
}

#[derive(Error, Debug)]
pub enum CommandParseError {
    #[error("Unknown Command")]
    UnknownCommand,
    #[error("Empty message buffer")]
    EmptyMessage,
    #[error("Incomplete args")]
    IncompleteArgs,
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    Utf8Error(#[from] Utf8Error),
}

impl CommandParseError {
    pub fn as_bytes(&self) -> &[u8] {
        use CommandParseError::*;
        match self {
            EmptyMessage => b"err;500",
            UnknownCommand => b"err;501",
            IncompleteArgs => b"err;502",
            ParseIntError(_) => b"err;503",
            Utf8Error(_) => b"err;504",
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::communication::eps::CommandParseError::*;

    #[test]
    fn empty_message_as_bytes() {
        assert_eq!(b"err;500", EmptyMessage.as_bytes());
    }

    #[test]
    fn unknown_command_as_bytes() {
        assert_eq!(b"err;501", UnknownCommand.as_bytes());
    }

    #[test]
    fn incomplete_args_as_bytes() {
        assert_eq!(b"err;502", IncompleteArgs.as_bytes());
    }

    #[test]
    fn parse_int_error_as_bytes() {
        let error = "test".parse::<u32>().err().unwrap();
        assert_eq!(b"err;503", ParseIntError(error).as_bytes());
    }

    #[test]
    fn utf_8_error_as_bytes() {
        #[allow(invalid_from_utf8)]
        let error = core::str::from_utf8(&[0xC0]).err().unwrap();
        assert_eq!(b"err;504", Utf8Error(error).as_bytes());
    }
}
