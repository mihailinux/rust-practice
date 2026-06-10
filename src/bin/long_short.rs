#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_time::{Timer, Instant}; //instant mandatory
use panic_probe as _;
use embassy_sync::{
    blocking_mutex::raw::ThreadModeRawMutex,
    channel::Channel,
};

#[derive(Clone, Copy)]
enum Length {
    Short,
    Long
}
//ex3_1_v2 de la async are ceva asemanator
static CHANNEL: Channel<ThreadModeRawMutex, Length, 64> = Channel::new();


#[embassy_executor::task]
async fn button_press(mut button: ExtiInput<'static>)
{
    loop{
        button.wait_for_falling_edge().await;
        let timestamp = Instant::now();
        button.wait_for_rising_edge().await;
        let duration = timestamp.elapsed().as_millis();

        if duration < 500 {
            CHANNEL.send(Length::Short).await;
        }
        else {
            CHANNEL.send(Length::Long).await;
        }
        Timer::after_millis(200).await;
    }
}

#[embassy_executor::task]
async fn led_control(mut led1: Output<'static>, mut led2: Output<'static>)
{
    loop{
        let value = CHANNEL.receive().await;

        match value {
            Length::Short => led1.set_high(),
            Length::Long => led2.set_high(),
        }

        Timer::after_millis(500).await;
        led1.set_low();
        led2.set_low();
        Timer::after_millis(500).await;
    }

}


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_stm32::init(Default::default());

    let button = ExtiInput::new(peripherals.PA0, peripherals.EXTI0, Pull::Up);
    let green = Output::new(peripherals.PC7, Level:: Low, Speed:: Low);
    let red = Output::new(peripherals.PC6, Level:: Low, Speed:: Low);
    
    spawner.spawn(button_press(button)).unwrap();
    spawner.spawn(led_control(green, red)).unwrap();

    loop{
        Timer::after_millis(200).await;
    }
}