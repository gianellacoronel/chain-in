use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Signer is not the profile authority")]
    UnauthorizedProfile,
    #[msg("Signer is not the habit owner")]
    UnauthorizedHabit,
    #[msg("Habit name exceeds the maximum byte length")]
    HabitNameTooLong,
    #[msg("Habit description exceeds the maximum byte length")]
    HabitDescriptionTooLong,
    #[msg("Habit ID overflow")]
    HabitIdOverflow,
    #[msg("Only active habits can receive check-ins")]
    HabitNotActive,
    #[msg("Check-in day does not match the current UTC day")]
    InvalidCheckInDay,
    #[msg("Streak counter overflow")]
    StreakOverflow,
    #[msg("Only active habits can be paused")]
    HabitNotPauseable,
    #[msg("Only paused habits can be resumed")]
    HabitNotResumable,
    #[msg("Only active or paused habits can be archived")]
    HabitNotArchivable,
    #[msg("Only archived habits can be closed")]
    HabitNotClosable,
}
