#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{/*AnyPin,*/ Level, Output, /*Pull,*/ Speed};
use embassy_time::Timer;
use panic_probe as _;
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use defmt::info;

static SIG: Signal<CriticalSectionRawMutex, u64> = Signal::new();

#[embassy_executor::task]
async fn pwm_generator()
{
    loop {
       for pwm in 1..=100 {
            info!("{}", pwm);
            SIG.signal(pwm);
            Timer::after_millis(10).await;
       }
       for pwm in (1..=100).rev() {
            info!("{}", pwm);
            SIG.signal(pwm);
            Timer::after_millis(10).await;
       }
    }
}

#[embassy_executor::task]
async fn led_control(mut led: Output<'static>)
{
    loop {
        let value = SIG.wait().await;
        for _ in 1..100{
            led.set_high();
            Timer::after_micros(value).await;
            led.set_low();
            Timer::after_micros(100-value).await; //duty cycle     
        }
    }
}



#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_stm32::init(Default::default());

    let led = Output:: new(peripherals.PC7, Level::Low, Speed::Low);

    spawner.spawn(pwm_generator()).unwrap();
    spawner.spawn(led_control(led)).unwrap();

    loop {
        Timer::after_millis(200).await;
    }

}