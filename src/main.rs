#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_stm32::{Config, exti::Channel, gpio::Pin};
use embassy_time::Timer;

use panic_reset as _;
use tasks::current_monitoring;

mod tasks;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    macro_rules! bind_oc_task {
        ($pin:expr, $channel:expr, $channel_num:literal) => {
            let pin = $pin.degrade();
            let ch = $channel.degrade();

            // TODO: MAKE SURE PULL IS CORRECT
            spawner.must_spawn(current_monitoring::watch_oc(pin, ch, $channel_num));
        };
    }

    bind_oc_task!(p.PC0, p.EXTI0, 0);

    loop {
        Timer::after_millis(2).await;
    }
}
