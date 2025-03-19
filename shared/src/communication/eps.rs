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
    use super::EpsCommand::{self, *};

    // EpsCommand parsing tests
    #[test]
    fn enable_power_rail() {
        assert_eq!(EnablePowerRail(0), EpsCommand::from_bytes(b"pwe;0").unwrap());
        assert_eq!(EnablePowerRail(1), EpsCommand::from_bytes(b"pwe;1").unwrap());
        assert_eq!(EnablePowerRail(2), EpsCommand::from_bytes(b"pwe;2").unwrap());
        assert_eq!(EnablePowerRail(3), EpsCommand::from_bytes(b"pwe;3").unwrap());
        assert_eq!(EnablePowerRail(4), EpsCommand::from_bytes(b"pwe;4").unwrap());
        assert_eq!(EnablePowerRail(5), EpsCommand::from_bytes(b"pwe;5").unwrap());
        assert_eq!(EnablePowerRail(6), EpsCommand::from_bytes(b"pwe;6").unwrap());
        assert_eq!(EnablePowerRail(7), EpsCommand::from_bytes(b"pwe;7").unwrap());
    }

    #[test]
    fn disable_power_rail() {
        assert_eq!(DisablePowerRail(0), EpsCommand::from_bytes(b"pwd;0").unwrap());
        assert_eq!(DisablePowerRail(1), EpsCommand::from_bytes(b"pwd;1").unwrap());
        assert_eq!(DisablePowerRail(2), EpsCommand::from_bytes(b"pwd;2").unwrap());
        assert_eq!(DisablePowerRail(3), EpsCommand::from_bytes(b"pwd;3").unwrap());
        assert_eq!(DisablePowerRail(4), EpsCommand::from_bytes(b"pwd;4").unwrap());
        assert_eq!(DisablePowerRail(5), EpsCommand::from_bytes(b"pwd;5").unwrap());
        assert_eq!(DisablePowerRail(6), EpsCommand::from_bytes(b"pwd;6").unwrap());
        assert_eq!(DisablePowerRail(7), EpsCommand::from_bytes(b"pwd;7").unwrap());
    }

    #[test]
    fn state_of_health() {
        assert_eq!(StateOfHealthReq, EpsCommand::from_bytes(b"soh").unwrap());
    }

    #[test]
    fn get_battery_voltage() {
        assert_eq!(GetBatteryVoltage(0), EpsCommand::from_bytes(b"gbv;0").unwrap());
        assert_eq!(GetBatteryVoltage(1), EpsCommand::from_bytes(b"gbv;1").unwrap());
    }

    #[test]
    fn get_power_rail_state() {
        assert_eq!(GetPowerRailState(0), EpsCommand::from_bytes(b"gprs;0").unwrap());
        assert_eq!(GetPowerRailState(1), EpsCommand::from_bytes(b"gprs;1").unwrap());
        assert_eq!(GetPowerRailState(2), EpsCommand::from_bytes(b"gprs;2").unwrap());
        assert_eq!(GetPowerRailState(3), EpsCommand::from_bytes(b"gprs;3").unwrap());
        assert_eq!(GetPowerRailState(4), EpsCommand::from_bytes(b"gprs;4").unwrap());
        assert_eq!(GetPowerRailState(5), EpsCommand::from_bytes(b"gprs;5").unwrap());
        assert_eq!(GetPowerRailState(6), EpsCommand::from_bytes(b"gprs;6").unwrap());
        assert_eq!(GetPowerRailState(7), EpsCommand::from_bytes(b"gprs;7").unwrap());
    }

    // CommandParseError tests 
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
