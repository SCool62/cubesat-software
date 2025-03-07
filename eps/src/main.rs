#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use embassy_executor::Spawner;
use embassy_stm32::Config;
use tasks::current_monitoring::spawn_oc_task;

use {panic_reset as _, defmt_rtt as _};

mod tasks;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    let _rail0_signal = spawn_oc_task(&spawner, p.PC0, p.EXTI0, 0).unwrap();
    let _rail1_signal = spawn_oc_task(&spawner, p.PC1, p.EXTI1, 1).unwrap();

}
