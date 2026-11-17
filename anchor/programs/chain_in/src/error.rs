use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Signer is not the profile authority")]
    UnauthorizedProfile,
    #[msg("Habit name exceeds the maximum byte length")]
    HabitNameTooLong,
    #[msg("Habit description exceeds the maximum byte length")]
    HabitDescriptionTooLong,
    #[msg("Habit ID overflow")]
    HabitIdOverflow,
}
