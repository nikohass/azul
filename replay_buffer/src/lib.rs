pub mod api;
pub mod buffer;
pub(crate) mod client;

pub const DEFAULT_BUFFER_SIZE: usize = 100_000;

use async_mutex::Mutex;
use buffer::Buffer;
use std::sync::Arc;

lazy_static::lazy_static! {
    pub static ref BUFFER: Arc<Mutex<Buffer>> = Buffer::with_size(DEFAULT_BUFFER_SIZE);
}

pub use client::ReplayBufferClient;
