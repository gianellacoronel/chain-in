use anchor_lang::prelude::*;

#[constant]
pub const PROFILE_SEED: &[u8] = b"profile";

#[constant]
pub const HABIT_SEED: &[u8] = b"habit";

#[constant]
pub const CHECKIN_SEED: &[u8] = b"checkin";

pub const MAX_HABIT_NAME_LEN: usize = 40;
pub const MAX_HABIT_DESCRIPTION_LEN: usize = 160;

/// Seconds in a UTC day, used to derive day indexes from unix timestamps.
pub const SECONDS_PER_DAY: i64 = 86_400;
