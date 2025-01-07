mod decode;
#[cfg(feature = "encode")]
mod encode;

pub use decode::decode_bcn;
#[cfg(feature = "encode")]
pub use encode::encode_bcn;

// All BCN formats use 4x4 pixel blocks.
const BLOCK_WIDTH: usize = 4;
const BLOCK_HEIGHT: usize = 4;
const CHANNELS: usize = 4;
const ELEMENTS_PER_BLOCK: usize = BLOCK_WIDTH * BLOCK_HEIGHT * CHANNELS;

pub struct Bc1;
pub struct Bc2;
pub struct Bc3;
pub struct Bc4;
pub struct Bc4S;
pub struct Bc5;
pub struct Bc5S;
pub struct Bc6;
pub struct Bc7;
