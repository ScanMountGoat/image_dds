#[cfg(feature = "decode")]
mod decode;
#[cfg(feature = "encode")]
mod encode;

#[cfg(feature = "decode")]
pub use decode::rgba_from_bcn;
#[cfg(feature = "encode")]
pub use encode::bcn_from_rgba;

// All BCN formats use 4x4 pixel blocks.
const BLOCK_WIDTH: usize = 4;
const BLOCK_HEIGHT: usize = 4;
const CHANNELS: usize = 4;
const ELEMENTS_PER_BLOCK: usize = BLOCK_WIDTH * BLOCK_HEIGHT * CHANNELS;

pub struct Bc1;
pub struct Bc2;
pub struct Bc3;
pub struct Bc4;
pub struct Bc5;
pub struct Bc6;
pub struct Bc7;
