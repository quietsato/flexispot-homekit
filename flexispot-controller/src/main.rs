use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::system::DeviceInfo;
use rppal::uart::Uart;
// use rppal::gpio::Gpio;

const COMMAND_WAKEUP: &[u8] = &[0x9b, 0x06, 0x02, 0x00, 0x00, 0x6c, 0xa1, 0x9d];
const COMMAND_UP: &[u8] = &[0x9b, 0x06, 0x02, 0x01, 0x00, 0xfc, 0xa0, 0x9d];

fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", DeviceInfo::new()?.model());

    // println!("Init GPIO");
    // let mut gpio = Gpio::new()?.get(18)?.into_output();
    // println!("DONE");
    println!("Init UART");
    let mut uart = Uart::new(9600, rppal::uart::Parity::None, 8, 1)?;
    uart.set_read_mode(1, Duration::from_secs(1))?;
    uart.set_write_mode(true)?;
    println!("DONE");

    // gpio.set_low();
    // println!("GPIO 18: LOW");
    // thread::sleep(Duration::from_secs(1));
    // gpio.set_high();
    // println!("GPIO 18: HIGH");
    // thread::sleep(Duration::from_secs(1));
    // gpio.set_low();
    // println!("GPIO 18: LOW");

    println!("UART send BEGIN");
    uart.write(&COMMAND_WAKEUP)?;
    println!("UART send END");
    println!("UART send BEGIN");
    uart.write(&COMMAND_UP)?;
    println!("UART send END");

    println!("read");
    let mut read = vec![0u8; 16];
    // current height: begins with `0x9b`, ends with `0x9d`

    loop {
        if uart.read(&mut read)? > 0 {
            for x in &read {
                print!("0x{x:0X}");
            }
            println!();
        }
        thread::sleep(Duration::from_secs(10));
        uart.write(&COMMAND_WAKEUP)?;
    }
}
