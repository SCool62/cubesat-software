use defmt::warn;
use embassy_futures::select::{select, Either};
use embassy_stm32::{mode::Async, usart::Uart};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, pubsub::{PubSubChannel, Publisher}};
use shared::communication::eps::EpsCommand;
use thiserror::Error;

use super::current_monitoring::{CurrentMonitorMessage, CURRENT_MONITOR_SIGNALS};

// Signals the EPS to send a message to the avionics board
pub static SEND_MESSAGE_CHANNEL: Channel<CriticalSectionRawMutex, &str, 1> = Channel::new();
// Sends read messages to all the tasks that may need it
// Make sure this size makes sense
pub static RECIEVED_MESSAGES_CHANNEL: PubSubChannel<CriticalSectionRawMutex, EpsCommand, 4, 4, 1> = PubSubChannel::new();

#[embassy_executor::task]
pub async fn avionics_communication(uart: &'static mut Uart<'static, Async>) {
    // The sole publisher to that channel
    let publisher = RECIEVED_MESSAGES_CHANNEL.publisher().unwrap();

    loop {
        let mut buf: [u8; 8] = [0; 8]; 
        match select(uart.read(&mut buf), SEND_MESSAGE_CHANNEL.receive()).await {
            // The UART recieved a message
            Either::First(r) => {
                match r {
                    Ok(_) => {
                        match EpsCommand::from_bytes(&buf) {
                            Ok(c) => {
                                match handle_message(&c, &publisher).await {
                                    // TODO: Respond to avionics board
                                    Ok(resp) => {
                                        todo!()
                                    },
                                    Err(e) => {
                                        todo!()
                                    }
                                }
                            },
                            Err(_e) => todo!() // TODO: Send response to avionics
                        }
                    },
                    Err(e) => warn!("Error reading UART: {}", e)
                }
            },
            // The signal recieved a signal
            Either::Second(m) => {
                // TODO: ACTUALLY HANDLE ERROR
                uart.write(m.as_bytes()).await.expect("Error writing UART");
            }
        } 
    }
}

async fn handle_message<'a>(command: &EpsCommand, publisher: &Publisher<'_, CriticalSectionRawMutex, EpsCommand, 4, 4, 1>) -> Result<Option<&'a str>, MessageHandleError> {
    match command {
        // If the command involves a power rail, signal that rail
        EpsCommand::EnablePowerRail(n) => {
            if let Some((signal,_)) = CURRENT_MONITOR_SIGNALS.get(n) {
                signal.signal(CurrentMonitorMessage::Activate);
                return Ok(None);
            } else {
                return Err(MessageHandleError::PowerRailNotFound);
            }
        },
        EpsCommand::DisablePowerRail(n) => {
            if let Some((signal, _)) = CURRENT_MONITOR_SIGNALS.get(n) {
                signal.signal(CurrentMonitorMessage::Deactivate);
                return Ok(None);
            } else {
                return Err(MessageHandleError::PowerRailNotFound);
            }
        },
        EpsCommand::GetPowerRailState(n) => {
            if let Some((_, watch)) = CURRENT_MONITOR_SIGNALS.get(n) {
                if watch.try_get().unwrap() {
                    return Ok(Some("On"));
                } else {
                    return Ok(Some("Off"));
                }
            } else {
                return Err(MessageHandleError::PowerRailNotFound);
            }
        }
        // If the command doesn't message the whole system
        _ => publisher.publish(*command).await
    }
    Ok(None)
}

#[derive(Error, Debug, Clone, Copy, PartialEq)]
enum MessageHandleError {
    #[error("Power Rail Not Found")]
    PowerRailNotFound
}