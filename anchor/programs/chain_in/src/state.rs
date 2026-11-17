use anchor_lang::prelude::*;

use crate::constants::{MAX_HABIT_DESCRIPTION_LEN, MAX_HABIT_NAME_LEN};

#[account]
#[derive(InitSpace)]
pub struct Profile {
    pub authority: Pubkey,
    pub next_habit_id: u64,
    pub created_at: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum HabitStatus {
    Active,
    Paused,
    Archived,
}

#[account]
#[derive(InitSpace)]
pub struct Habit {
    pub owner: Pubkey,
    pub id: u64,
    #[max_len(MAX_HABIT_NAME_LEN)]
    pub name: String,
    #[max_len(MAX_HABIT_DESCRIPTION_LEN)]
    pub description: String,
    pub status: HabitStatus,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub last_check_in_day: Option<i64>,
    pub created_at: i64,
}

#[account]
#[derive(InitSpace)]
pub struct DailyCheckIn {
    pub owner: Pubkey,
    pub habit: Pubkey,
    pub habit_id: u64,
    #[max_len(MAX_HABIT_NAME_LEN)]
    pub habit_name_snapshot: String,
    pub day: i64,
    pub checked_at: i64,
}
