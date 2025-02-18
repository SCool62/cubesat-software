#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

use panic_reset as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    loop {
        Timer::after_millis(2).await;
    }
}
