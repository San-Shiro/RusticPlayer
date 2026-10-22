pub mod tokens;
pub mod icon;
pub mod atoms;
pub mod molecules;
pub mod organisms;
pub mod mock;

use std::sync::OnceLock;
use std::time::Instant;

pub fn anim_epoch() -> Instant {
    static EPOCH: OnceLock<Instant> = OnceLock::new();
    *EPOCH.get_or_init(Instant::now)
}
