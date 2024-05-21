use std::error::Error;
use std::thread;
use std::time::Duration;

use rppal::system::DeviceInfo;
use rppal::uart::Uart;

const COMMAND_WAKEUP: &[u8] = &[0x9b, 0x06, 0x02, 0x00, 0x00, 0x6c, 0xa1, 0x9d];
const COMMAND_UP: &[u8] = &[0x9b, 0x06, 0x02, 0x01, 0x00, 0xfc, 0xa0, 0x9d];

fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", DeviceInfo::new()?.model());

    println!("Init UART");
    let mut uart = Uart::new(9600, rppal::uart::Parity::None, 8, 1)?;
    uart.set_read_mode(1, Duration::from_secs(1))?;
    uart.set_write_mode(true)?;
    println!("DONE");

    println!("UART send BEGIN");
    uart.write(&COMMAND_WAKEUP)?;
    println!("UART send END");
    thread::sleep(Duration::from_millis(500)); // friendly sleep
    println!("UART send BEGIN");
    uart.write(&COMMAND_UP)?;
    println!("UART send END");

    println!("read");
    let mut read = vec![0u8; 16];
    // current height: begins with `0x9b`, ends with `0x9d`

    loop {
        let len = uart.read(&mut read)?;
        if len > 0 {
            for x in &read[..len] {
                print!("0x{x:0X}");
            }
            println!();
        }
        thread::sleep(Duration::from_secs(10));
        uart.write(&COMMAND_WAKEUP)?;
        thread::sleep(Duration::from_millis(500)); // friendly sleep
        uart.write(&COMMAND_UP)?;
    }
}
