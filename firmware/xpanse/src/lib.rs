#![no_std]

use defmt::info;
use heapless::String;
pub struct Hello {
    pub message: String<32>,
}

impl Hello {
    pub fn new(message: String<32>) -> Self {
        Self { message }
    }

    pub fn say(&self) {
        info!("{}", self.message.as_str());
    }
}
