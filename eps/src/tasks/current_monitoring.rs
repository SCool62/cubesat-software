use embassy_executor::{SpawnError, Spawner};
use embassy_futures::select::{Either, select};
use embassy_stm32::{
    exti::{AnyChannel, Channel, ExtiInput},
    gpio::{AnyPin, Pin, Pull},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal};
use phf::phf_map;

#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrentMonitorMessage {
    Activate,
    Deactivate,
    OcEvent,
}

macro_rules! define_cm_signal {
    ($ch:ident) => {
        static $ch: Signal<ThreadModeRawMutex, CurrentMonitorMessage> = Signal::new();
    };
}

define_cm_signal!(CM_0);
define_cm_signal!(CM_1);
define_cm_signal!(CM_2);
define_cm_signal!(CM_3);
define_cm_signal!(CM_4);
define_cm_signal!(CM_5);
define_cm_signal!(CM_6);
define_cm_signal!(CM_7);

pub static CURRENT_MONITOR_SIGNALS: phf::Map<
    u8,
    &'static Signal<ThreadModeRawMutex, CurrentMonitorMessage>,
> = phf_map! {
    0u8 => &CM_0,
    1u8 => &CM_1,
    2u8 => &CM_2,
    3u8 => &CM_3,
    4u8 => &CM_4,
    5u8 => &CM_5,
    6u8 => &CM_6,
    7u8 => &CM_7
};

// TODO: Make sure pool size is the amount of current sensing circuits
// PANICS if signal_channel doesn't exist
#[embassy_executor::task(pool_size = 8)]
pub async fn watch_oc(exti_pin: AnyPin, exti_channel: AnyChannel, signal_num: u8) {
    // TODO: SHOULD THIS BE Pull::Down?
    let mut interrupt = ExtiInput::new(exti_pin, exti_channel, Pull::Down);
    // Expect is OK here because the task doesn't work without its apropriate signal channel
    let signal = CURRENT_MONITOR_SIGNALS
        .get(&signal_num)
        .expect("That signal doesn't exist!");

    loop {
        // TODO: MAKE SURE THIS MAKES SENSE
        match select(interrupt.wait_for_falling_edge(), signal.wait()).await {
            // TODO: HANDLE
            // The interrupt was triggered
            Either::First(_) => todo!(),
            // The signal was recieved
            Either::Second(message) => match message {
                CurrentMonitorMessage::Activate => todo!(),
                CurrentMonitorMessage::Deactivate => todo!(),
                CurrentMonitorMessage::OcEvent => (),
            },
        };
    }
}

// Spawns the task and returns the Signal for the task or a SpawnError
pub fn spawn_oc_task(
    spawner: &Spawner,
    exti_pin: impl Pin,
    exti_channel: impl Channel,
    signal_num: u8,
) -> Result<&'static Signal<ThreadModeRawMutex, CurrentMonitorMessage>, SpawnError> {
    spawner.spawn(watch_oc(
        exti_pin.degrade(),
        exti_channel.degrade(),
        signal_num,
    ))?;
    // Unwrap is appropriate because the task will fail with a message
    Ok(CURRENT_MONITOR_SIGNALS.get(&signal_num).unwrap())
}
