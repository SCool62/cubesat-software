use core::f64;

use concat_idents::concat_idents;
use embassy_executor::{SpawnError, Spawner};
use embassy_futures::select::{Either, select};
use embassy_stm32::{
    exti::{AnyChannel, Channel, ExtiInput},
    gpio::{AnyPin, OutputOpenDrain, Pin, Pull},
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
    ($($ch:ident),*) => {
        $(
            concat_idents!(reciever_name = $ch, R {
                static reciever_name: Signal<ThreadModeRawMutex, CurrentMonitorMessage> = Signal::new();
            });
            concat_idents!(sender_name = $ch, S {
                static sender_name: Watch<ThreadModeRawMutex, bool, 2> = Watch::new_with(true);
            });
        )*
    }
}

define_cm_signals! { CM_0, CM_1, CM_2, CM_3, CM_4, CM_5, CM_6, CM_7 }

type CurrentMonitorSignal = (
    &'static Signal<ThreadModeRawMutex, CurrentMonitorMessage>,
    &'static Watch<ThreadModeRawMutex, bool, 2>,
);

pub static CURRENT_MONITOR_SIGNALS: phf::Map<u8, CurrentMonitorSignal> = phf_map! {
    0u8 => (&CM_0R, &CM_0S),
    1u8 => (&CM_1R, &CM_1S),
    2u8 => (&CM_2R, &CM_2S),
    3u8 => (&CM_3R, &CM_3S),
    4u8 => (&CM_4R, &CM_4S),
    5u8 => (&CM_5R, &CM_5S),
    6u8 => (&CM_6R, &CM_6S),
    7u8 => (&CM_7R, &CM_7S)
};

// PANICS if signal_channel doesn't exist
#[embassy_executor::task(pool_size = CURRENT_MONITOR_SIGNALS.len())]
pub async fn watch_oc(
    output_pin: AnyPin,
    exti_pin: AnyPin,
    exti_channel: AnyChannel,
    signal_num: u8,
) {
    let mut interrupt = ExtiInput::new(exti_pin, exti_channel, Pull::Up);

    // Leaves pin floating while set to high
    let mut output_pin =
        OutputOpenDrain::new(output_pin, true.into(), embassy_stm32::gpio::Speed::Low);

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
                sender.send(interrupt.get_level().into());
                // Latch the rail value
                output_pin.set_level(interrupt.get_level());
            }
            // The signal was recieved
            Either::Second(message) => match message {
                CurrentMonitorMessage::Activate => output_pin.set_high(),
                CurrentMonitorMessage::Deactivate => output_pin.set_low(),
            },
        };
    }
}

// Spawns the task and returns the Signal for the task or a SpawnError
pub fn spawn_oc_task(
    spawner: &Spawner,
    output_pin: impl Pin,
    exti_pin: impl Pin,
    exti_channel: impl Channel,
    signal_num: u8,
) -> Result<&'static Signal<ThreadModeRawMutex, CurrentMonitorMessage>, SpawnError> {
    spawner.spawn(watch_oc(
        output_pin.degrade(),
        exti_pin.degrade(),
        exti_channel.degrade(),
        signal_num,
    ))?;
    // Unwrap is appropriate because the task will fail with a message
    Ok(CURRENT_MONITOR_SIGNALS.get(&signal_num).unwrap().0)
}
