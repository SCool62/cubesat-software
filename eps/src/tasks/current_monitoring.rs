use concat_idents::concat_idents;
use embassy_executor::{SpawnError, Spawner};
use embassy_futures::select::{Either, select};
use embassy_stm32::{
    exti::{AnyChannel, Channel, ExtiInput},
    gpio::{AnyPin, Pin, Pull},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, signal::Signal, watch::Watch};
use phf::phf_map;

#[allow(unused)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CurrentMonitorMessage {
    Activate,
    Deactivate,
}

macro_rules! define_cm_signals {
    ($ch:ident) => {
        concat_idents!(reciever_name = $ch, R {
            static reciever_name: Signal<ThreadModeRawMutex, CurrentMonitorMessage> = Signal::new();
        });
        concat_idents!(sender_name = $ch, S {
            static sender_name: Watch<ThreadModeRawMutex, bool, 2> = Watch::new_with(true);
        });
    };
}

define_cm_signals!(CM_0);
define_cm_signals!(CM_1);
define_cm_signals!(CM_2);
define_cm_signals!(CM_3);
define_cm_signals!(CM_4);
define_cm_signals!(CM_5);
define_cm_signals!(CM_6);
define_cm_signals!(CM_7);

type CurrentMonitorSignal = (&'static Signal<ThreadModeRawMutex, CurrentMonitorMessage>, &'static Watch<ThreadModeRawMutex, bool, 2>);

pub static CURRENT_MONITOR_SIGNALS: phf::Map<
    u8,
    CurrentMonitorSignal,
> = phf_map! {
    0u8 => (&CM_0R, &CM_0S),
    1u8 => (&CM_1R, &CM_1S),
    2u8 => (&CM_2R, &CM_2S),
    3u8 => (&CM_3R, &CM_3S),
    4u8 => (&CM_4R, &CM_4S),
    5u8 => (&CM_5R, &CM_5S),
    6u8 => (&CM_6R, &CM_6S),
    7u8 => (&CM_7R, &CM_7S)
};

// TODO: Make sure pool size is the amount of current sensing circuits
// PANICS if signal_channel doesn't exist
#[embassy_executor::task(pool_size = 8)]
pub async fn watch_oc(exti_pin: AnyPin, exti_channel: AnyChannel, signal_num: u8) {
    // TODO: SHOULD THIS BE Pull::Down?
    let mut interrupt = ExtiInput::new(exti_pin, exti_channel, Pull::Down);
    // Expect is OK here because the task doesn't work without its apropriate signal channel
    let (signal, watch) = CURRENT_MONITOR_SIGNALS
        .get(&signal_num)
        .expect("That signal doesn't exist!");
    let sender = watch.sender();
    sender.send(interrupt.is_high());

    loop {
        // TODO: MAKE SURE THIS MAKES SENSE
        match select(interrupt.wait_for_falling_edge(), signal.wait()).await {
            // The interrupt was triggered
            Either::First(_) => {
                sender.send(interrupt.is_high());
                // TODO: HANDLE
            }
            // The signal was recieved
            Either::Second(message) => match message {
                CurrentMonitorMessage::Activate => todo!(),
                CurrentMonitorMessage::Deactivate => todo!(),
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
    Ok(CURRENT_MONITOR_SIGNALS.get(&signal_num).unwrap().0)
}
