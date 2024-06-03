use std::time::Duration;

use log::info;

use crate::flexispot::command::{FlexispotCommand, FlexispotCommandExecutor, FlexispotPreset};
use crate::flexispot::query::{FlexispotPacket, FlexispotQueryProcessor, FlexispotQueryResult};

pub trait FlexispotMqttAdapter {
    fn get_desk_height(&self) -> Option<f32>;
    fn wakeup(&self);
    fn friendly_sleep(&self);
    fn call_preset(&self, preset: &FlexispotPreset);
}

#[derive(Debug, Default)]
pub struct FlexispotMqttAdapterMock;

impl FlexispotMqttAdapter for FlexispotMqttAdapterMock {
    fn get_desk_height(&self) -> Option<f32> {
        None
    }

    fn wakeup(&self) {}

    fn friendly_sleep(&self) {}

    fn call_preset(&self, preset: &FlexispotPreset) {
        info!("called preset {preset:?}");
    }
}

pub struct FlexispotMqttAdapterImpl {
    command_executor: FlexispotCommandExecutor,
    query_processor: FlexispotQueryProcessor,
    sleep_duration: Duration,
}

impl FlexispotMqttAdapterImpl {
    pub fn new(
        command_executor: FlexispotCommandExecutor,
        query_processor: FlexispotQueryProcessor,
        sleep_duration: Duration,
    ) -> Self {
        Self {
            command_executor,
            query_processor,
            sleep_duration,
        }
    }
}

impl FlexispotMqttAdapter for FlexispotMqttAdapterImpl {
    fn get_desk_height(&self) -> Option<f32> {
        let mut buf = vec![0; 512];
        self.query_processor.read(&mut buf).unwrap();
        let packets = FlexispotQueryResult::new(&buf).parse();
        packets
            .iter()
            .rev()
            .filter_map(|packet| match packet {
                FlexispotPacket::Unknown => None,
                FlexispotPacket::Sleep => None,
                FlexispotPacket::CurrentHeight(height) => Some(height.to_f32()),
            })
            .next()
    }

    fn wakeup(&self) {
        self.command_executor
            .execute(FlexispotCommand::Wakeup)
            .unwrap();
    }

    fn friendly_sleep(&self) {
        self.command_executor.sleep(self.sleep_duration);
    }

    fn call_preset(&self, preset: &FlexispotPreset) {
        let command = FlexispotCommand::from(preset);
        self.command_executor.execute(command).unwrap();
    }
}
