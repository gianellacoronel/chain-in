use anchor_lang::prelude::*;

use crate::{Habit, HabitStatus, constants::*, error::ErrorCode};

#[derive(Accounts)]
pub struct CloseHabit<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [HABIT_SEED, owner.key().as_ref(), &habit.id.to_le_bytes()],
        bump,
        has_one = owner @ ErrorCode::UnauthorizedHabit,
        constraint = habit.status == HabitStatus::Archived @ ErrorCode::HabitNotClosable,
        close = owner
    )]
    pub habit: Account<'info, Habit>,
}

pub fn handle_close_habit(_ctx: Context<CloseHabit>) -> Result<()> {
    Ok(())
}
