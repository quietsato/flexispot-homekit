use std::time::Duration;

use log::info;

use crate::flexispot::command::{FlexispotCommand, FlexispotCommandExecutor, FlexispotPreset};
use crate::flexispot::query::{FlexispotPacket, FlexispotQueryProcessor, FlexispotQueryResult};

pub trait FlexispotMqttAdaptor {
    fn get_desk_height(&self) -> Option<f32>;
    fn wakeup(&self);
    fn friendly_sleep(&self);
    fn call_profile(&self, preset: &FlexispotPreset);
}

#[derive(Debug, Default)]
pub struct FlexispotMqttAdaptorMock;

impl FlexispotMqttAdaptor for FlexispotMqttAdaptorMock {
    fn get_desk_height(&self) -> Option<f32> {
        None
    }

    fn wakeup(&self) {}

    fn friendly_sleep(&self) {}

    fn call_profile(&self, preset: &FlexispotPreset) {
        info!("called preset {preset:?}");
    }
}

pub struct FlexispotMqttAdaptorImpl {
    command_executor: FlexispotCommandExecutor,
    query_processor: FlexispotQueryProcessor,
    sleep_duration: Duration,
}

impl FlexispotMqttAdaptorImpl {
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

impl FlexispotMqttAdaptor for FlexispotMqttAdaptorImpl {
    fn get_desk_height(&self) -> Option<f32> {
        let mut buf = vec![];
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

    fn call_profile(&self, preset: &FlexispotPreset) {
        let command = FlexispotCommand::from(preset);
        self.command_executor.execute(command).unwrap();
    }
}
