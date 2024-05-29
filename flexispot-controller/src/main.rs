mod adapter;
mod flexispot;
mod mqtt;

use std::borrow::BorrowMut;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use dotenv::dotenv;
use log::info;
use rppal::system::DeviceInfo;
use rppal::uart::{Parity, Uart};
use rumqttc::MqttOptions;

use crate::adapter::FlexispotMqttAdapterImpl;
use crate::flexispot::command::FlexispotCommandExecutor;
use crate::flexispot::query::FlexispotQueryProcessor;
use crate::mqtt::controller::MqttController;

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    println!("{}", DeviceInfo::new()?.model());

    println!("Init UART");
    let mut uart = Uart::new(9600, Parity::None, 8, 1)?;
    uart.set_read_mode(1, Duration::from_secs(1))?;
    uart.borrow_mut().set_write_mode(false)?;
    let uart = Arc::new(Mutex::new(uart));
    let command_executor = FlexispotCommandExecutor::new(Arc::clone(&uart));
    let query_processor = FlexispotQueryProcessor::new(Arc::clone(&uart));
    println!("DONE");

    let adapter = FlexispotMqttAdapterImpl::new(
        command_executor,
        query_processor,
        Duration::from_millis(500),
    );

    // configure mqtt
    let mut mqtt_options = {
        let host = std::env::var("MQTT_HOST").unwrap();
        let port = std::env::var("MQTT_PORT").unwrap().parse().unwrap();
        info!("MQTT host: {host}");
        info!("MQTT port: {port}");
        MqttOptions::new("flexispot-homekit", host, port)
    };
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    mqtt_options.set_transport(rumqttc::Transport::Tcp);

    let controller = MqttController::new(mqtt_options, Arc::new(Mutex::new(adapter)));

    controller.run()?;

    Ok(())
}
