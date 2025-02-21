#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_stm32::{exti::Channel, gpio::{Pin, Pull}, Config};
use embassy_time::Timer;

use panic_reset as _;
use tasks::respond_oc;

mod tasks;



#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    macro_rules! bind_oc_task {
        ($pin:expr, $channel:expr) => {
            let pin = $pin.degrade();
            let ch = $channel.degrade();
            

            // TODO: MAKE SURE PULL IS CORRECT
            spawner.must_spawn(respond_oc::watch_oc(pin, ch, Pull::Down));
        };
    }

    bind_oc_task!(p.PC13, p.EXTI13);

    loop {
        Timer::after_millis(2).await;
    }
}
