#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use core::time::Duration;

use embassy_executor::Spawner;
use embassy_stm32::{peripherals::IWDG, wdg::IndependentWatchdog, Config};
use embassy_time::Timer;
use tasks::current_monitoring::spawn_oc_task;

use {panic_reset as _, defmt_rtt as _};

mod tasks;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    spawner.must_spawn(watchdog(p.IWDG));

    let _rail0_signal = spawn_oc_task(&spawner, p.PC0, p.EXTI0, 0).unwrap();
    let _rail1_signal = spawn_oc_task(&spawner, p.PC1, p.EXTI1, 1).unwrap();

}

#[embassy_executor::task]
async fn watchdog(wdg: IWDG) {
    // Watchdog with a 20 second timeout
    let mut watchdog = IndependentWatchdog::new(wdg, Duration::from_secs(20).as_micros() as u32);
    // Start watchdog
    watchdog.unleash();

    loop {
        watchdog.pet();
        // Wait 15 seconds before petting again
        Timer::after_secs(15).await;
    }
}