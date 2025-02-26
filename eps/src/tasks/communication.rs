use defmt::warn;
use embassy_stm32::{mode::Async, usart::Uart};

#[embassy_executor::task]
pub async fn avionics_communication(uart: &'static mut Uart<'static, Async>) {
    loop {
        let mut buf: [u8; 8] = [0; 8]; 
        match uart.read(&mut buf).await {
            Ok(_) => {
                todo!();
            },
            Err(e) => warn!("Error reading UART: {}", e)
        }
    }
}
