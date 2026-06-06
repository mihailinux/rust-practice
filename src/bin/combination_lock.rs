#![no_std]
#![no_main]

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{/*AnyPin,*/ Level, Output, Pull, Speed};
use embassy_time::Timer;
use panic_probe as _;
use embassy_sync::channel::Channel;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;


static CHANNEL : Channel <ThreadModeRawMutex, bool, 10> = Channel::new();


#[embassy_executor::task]
async fn button1(mut button: ExtiInput<'static>){
    loop{
        button.wait_for_falling_edge().await;
        CHANNEL.send(true).await;
        Timer::after_millis(200).await;
    }
}

#[embassy_executor::task]
async fn button2(mut button: ExtiInput<'static>){
    loop{
        button.wait_for_falling_edge().await;
        CHANNEL.send(false).await;
        Timer::after_millis(200).await;
    }
}


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_stm32::init(Default::default());

    let first_button = ExtiInput::new(peripherals.PA0, peripherals.EXTI0, Pull::Up);
    let second_button = ExtiInput::new(peripherals.PA1, peripherals.EXTI1, Pull::Up);

    spawner.spawn(button1(first_button)).unwrap();
    spawner.spawn(button2(second_button)).unwrap();

    let mut red = Output::new(peripherals.PC6, Level::Low, Speed::Low);
    let mut green = Output::new(peripherals.PC7, Level::Low, Speed::Low);

    red.set_high();
    green.set_low();

    let sequence = [true, true, false, true];
    let mut step = 0;
    loop{
        let message = CHANNEL.receive().await;
        info!("received: {}", message);
        info!("expected: {}", sequence[step]);

        if message == sequence[step] {
            step = step + 1;
        }
        else {
            for _ in 0..3 {
                red.set_high();
                Timer::after_millis(100).await;
                red.set_low();
                Timer::after_millis(100).await;
            }
            step = 0;
            red.set_high();
        }

        if step == 4{
            red.set_low();
            green.set_high();
            Timer::after_secs(3).await;
            info!("Thats it!");
            red.set_high();
            green.set_low();
            step = 0;
        }

        Timer::after_millis(200).await;
    }
}