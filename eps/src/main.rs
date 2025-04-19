#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use core::time::Duration;

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts, mode::Async, peripherals::{self, IWDG}, usart::{self, Uart}, wdg::IndependentWatchdog, Config
};
use embassy_time::Timer;
use static_cell::StaticCell;
use tasks::{communication, current_monitoring::spawn_oc_task};

use {defmt_rtt as _, panic_reset as _};

mod tasks;

bind_interrupts!(struct Irqs {
    UART4 => usart::InterruptHandler<peripherals::UART4>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());

    spawner.must_spawn(watchdog(p.IWDG));

    let _rail0_signal = spawn_oc_task(&spawner, p.PB0, p.PC0, p.EXTI0, 0).unwrap();
    let _rail1_signal = spawn_oc_task(&spawner, p.PB1, p.PC1, p.EXTI1, 1).unwrap();

    {
        static UART4: StaticCell<Uart<'_, Async>> = StaticCell::new();
        spawner.must_spawn(communication::avionics_communication(
            UART4.init(
                Uart::new(
                    p.UART4,
                    p.PA1,
                    p.PA0,
                    Irqs,
                    p.DMA2_CH3,
                    p.DMA2_CH5,
                    Default::default(),
                )
                .expect("Error setting up UART"),
            ),
        ));
    }
}

#[embassy_executor::task]
async fn watchdog(wdg: IWDG) {
    // Watchdog with a 20 second timeout
    // Lossy cast is OK because 20 seconds as micros fits within the available range of a u32
    let mut watchdog = IndependentWatchdog::new(wdg, Duration::from_secs(20).as_micros() as u32);
    // Start watchdog
    watchdog.unleash();

    loop {
        watchdog.pet();
        // Wait 15 seconds before petting again
        Timer::after_secs(15).await;
    }
}
