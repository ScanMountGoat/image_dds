// All BCN formats use 4x4 pixel blocks.
const BLOCK_WIDTH: usize = 4;
const BLOCK_HEIGHT: usize = 4;

#[cfg(feature = "decode")]
mod decode;
#[cfg(feature = "encode")]
mod encode;

#[cfg(feature = "decode")]
pub use decode::rgba8_from_bcn;
#[cfg(feature = "encode")]
pub use encode::bcn_from_rgba8;

pub struct Bc1;
pub struct Bc2;
pub struct Bc3;
pub struct Bc4;
pub struct Bc5;
pub struct Bc6;
pub struct Bc7;
