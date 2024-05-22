mod flexispot;

use std::borrow::BorrowMut;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rppal::system::DeviceInfo;
use rppal::uart::{Parity, Uart};

use crate::flexispot::command::{FlexispotCommand, FlexispotCommandExecutor};
use crate::flexispot::query::FlexispotQueryProcessor;

fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", DeviceInfo::new()?.model());

    println!("Init UART");
    let mut uart = Uart::new(9600, Parity::None, 8, 1)?;
    uart.set_read_mode(1, Duration::from_secs(1))?;
    uart.borrow_mut().set_write_mode(true)?;

    let uart = Arc::new(Mutex::new(uart));
    let executor = FlexispotCommandExecutor::new(Arc::clone(&uart));
    let query_processor = FlexispotQueryProcessor::new(Arc::clone(&uart));

    println!("DONE");

    println!("UART send BEGIN");
    executor.execute(FlexispotCommand::Wakeup)?;
    println!("UART send END");
    executor.sleep(Duration::from_millis(500)); // friendly sleep
    println!("UART send BEGIN");
    executor.execute(FlexispotCommand::Preset4)?;
    println!("UART send END");

    println!("read");
    let mut buf = vec![0; 512];
    // current height: begins with `0x9b`, ends with `0x9d`

    loop {
        let len = query_processor.read(&mut buf)?;
        println!("Read {len} bytes");
        if len > 0 {
            for x in &buf[..len] {
                print!("0x{x:0X}");
            }
            println!();
        }
        thread::sleep(Duration::from_secs(5));
        executor.execute(FlexispotCommand::Wakeup)?;
        executor.sleep(Duration::from_millis(500));
    }
}
