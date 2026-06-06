#![no_std]
#![no_main]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{/*AnyPin,*/ Level, Output, Pull, Speed};
use embassy_time::Timer;
use panic_probe as _;
use embassy_sync::signal::Signal;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;



static SIG: Signal<CriticalSectionRawMutex, bool> = Signal::new();

#[embassy_executor::task]
async fn button_watch(button: ExtiInput<'static>){
    loop{
        let state = button.is_high();
        SIG.signal(state);
        Timer::after_millis(50).await;
    }
}

#[embassy_executor::task]
async fn counter_task(mut led1: Output<'static>, mut led2: Output<'static>, mut led3:Output<'static>){
    let mut counter: u8=0;
    loop{
        let value = SIG.wait().await;
        if value {
            counter = (counter + 1 ) & 0x07; //up
        }
        else {
            counter = counter.wrapping_sub(1)& 0x07; //down
        }
        if counter & 0b001 != 0 {led1.set_high();} else {led1.set_low();}
        if counter & 0b010 != 0 {led2.set_high();} else {led2.set_low();}
        if counter & 0b100 != 0 {led3.set_high();} else {led3.set_low();}

        Timer::after_secs(1).await;
    }
}
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = embassy_stm32::init(Default::default());

    let led1 = Output::new(peripherals.PC7, Level::Low, Speed::Medium);
    let led2 = Output::new(peripherals.PC6, Level::Low, Speed::Medium);
    let led3 = Output::new(peripherals.PC9, Level::Low, Speed::Medium); 
    let button1 = ExtiInput::new(peripherals.PA0, peripherals.EXTI0, Pull::Up);

    
    spawner.spawn(button_watch(button1)).unwrap();
    spawner.spawn(counter_task(led1, led2, led3)).unwrap();

    loop{
        Timer::after_millis(200).await;
    }
}