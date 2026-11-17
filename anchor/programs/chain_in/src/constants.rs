use anchor_lang::prelude::*;

#[constant]
pub const PROFILE_SEED: &[u8] = b"profile";

#[constant]
pub const HABIT_SEED: &[u8] = b"habit";

pub const MAX_HABIT_NAME_LEN: usize = 40;
pub const MAX_HABIT_DESCRIPTION_LEN: usize = 160;
