use std::{
    borrow::BorrowMut,
    error::Error,
    fmt::Display,
    sync::{Arc, Mutex},
    time::Duration,
};

#[rustfmt::skip]
mod constants {
    pub const COMMAND_WAKEUP:  &[u8] = &[0x9b, 0x06, 0x02, 0x00, 0x00, 0x6c, 0xa1, 0x9d];
    pub const COMMAND_UP:      &[u8] = &[0x9b, 0x06, 0x02, 0x01, 0x00, 0xfc, 0xa0, 0x9d];
    pub const COMMAND_DOWN:    &[u8] = &[0x9b, 0x06, 0x02, 0x02, 0x00, 0x0c, 0xa0, 0x9d];
    pub const COMMAND_M:       &[u8] = &[0x9b, 0x06, 0x02, 0x20, 0x00, 0xac, 0xb8, 0x9d];
    pub const COMMAND_PRESET1: &[u8] = &[0x9b, 0x06, 0x02, 0x04, 0x00, 0xac, 0xa3, 0x9d];
    pub const COMMAND_PRESET2: &[u8] = &[0x9b, 0x06, 0x02, 0x08, 0x00, 0xac, 0xa6, 0x9d];
    pub const COMMAND_PRESET3: &[u8] = &[0x9b, 0x06, 0x02, 0x10, 0x00, 0xac, 0xac, 0x9d];
    pub const COMMAND_PRESET4: &[u8] = &[0x9b, 0x06, 0x02, 0x00, 0x01, 0xac, 0x60, 0x9d];
}

#[derive(Debug)]
pub enum FlexispotPreset {
    Preset1,
    Preset2,
    Preset3,
    Preset4,
}

impl From<&FlexispotPreset> for FlexispotCommand {
    fn from(value: &FlexispotPreset) -> Self {
        match value {
            FlexispotPreset::Preset1 => Self::Preset1,
            FlexispotPreset::Preset2 => Self::Preset2,
            FlexispotPreset::Preset3 => Self::Preset3,
            FlexispotPreset::Preset4 => Self::Preset4,
        }
    }
}

#[derive(Debug)]
pub enum FlexispotCommand {
    Wakeup,
    Up,
    Down,
    M,
    Preset1,
    Preset2,
    Preset3,
    Preset4,
}

impl FlexispotCommand {
    pub fn to_u8_array(&self) -> &'static [u8] {
        match self {
            Self::Wakeup => constants::COMMAND_WAKEUP,
            Self::Up => constants::COMMAND_UP,
            Self::Down => constants::COMMAND_DOWN,
            Self::M => constants::COMMAND_M,
            Self::Preset1 => constants::COMMAND_PRESET1,
            Self::Preset2 => constants::COMMAND_PRESET2,
            Self::Preset3 => constants::COMMAND_PRESET3,
            Self::Preset4 => constants::COMMAND_PRESET4,
        }
    }
}

pub struct FlexispotCommandResponse {}

#[derive(Debug)]
pub struct FlexispotCommandError(pub String);
impl Display for FlexispotCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl Error for FlexispotCommandError {}

#[derive(Debug)]
pub struct FlexispotCommandExecutor {
    uart: Arc<Mutex<rppal::uart::Uart>>,
}

impl FlexispotCommandExecutor {
    pub fn new(uart: Arc<Mutex<rppal::uart::Uart>>) -> Self {
        Self { uart }
    }
    pub fn execute(
        &self,
        command: FlexispotCommand,
    ) -> Result<FlexispotCommandResponse, FlexispotCommandError> {
        self.uart
            .lock()
            .unwrap()
            .borrow_mut()
            .write(command.to_u8_array())
            .map_err(|e| FlexispotCommandError(e.to_string()))?;
        Ok(FlexispotCommandResponse {})
    }
    pub fn sleep(&self, dur: Duration) {
        std::thread::sleep(dur)
    }
}
