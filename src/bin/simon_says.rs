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


static CHANNEL : Channel <ThreadModeRawMutex, u8, 64> = Channel::new();

#[embassy_executor::task(pool_size=2)]
async fn button_press(mut button: ExtiInput<'static>, id: u8)
{
    loop {
        button.wait_for_falling_edge().await;
        info!("Press");
        CHANNEL.send(id).await;
        Timer::after_millis(200).await;
    }
}




#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_stm32::init(Default::default());

    let button1 = ExtiInput::new(peripherals.PA0, peripherals.EXTI0, Pull::Up);
    let button2 = ExtiInput::new(peripherals.PA1, peripherals.EXTI1, Pull::Up);

    let id1: u8 = 1;
    let id2: u8 = 2;

    spawner.spawn(button_press(button1, id1)).unwrap();
    spawner.spawn(button_press(button2, id2)).unwrap();

    let mut led1 = Output::new(peripherals.PC7, Level::Low, Speed::Low );
    let mut led2 = Output::new(peripherals.PC6, Level::Low, Speed::Low );

    let sequence = [1, 2, 1, 1, 2, 1, 2, 3];
    let mut length = 1;
    let mut correct: bool;
    loop{
        for state in 0..length{

            match sequence[state]{
                1=> {led1.set_high(); Timer::after_secs(1).await; led2.set_low();}
                2=> {led2.set_high(); Timer::after_secs(1).await; led1.set_low();}
                _=> {}
            }

            led1.set_low();
            led2.set_low();
        }

        led1.set_low();
        Timer::after_millis(200).await;
        led2.set_low();
        Timer::after_millis(200).await;

        correct = true;

        for i in 0..length{
            let value = CHANNEL.receive().await;
            if sequence[i] != value {
                correct = false;
                break;
            }
        }

        if correct {
            length = length + 1;
        }
        else {
            length = 1;
            led1.set_low();
            Timer::after_millis(200).await;
            led2.set_low();
            Timer::after_millis(200).await;
        }
    }

}