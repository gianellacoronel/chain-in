use anchor_lang::prelude::*;

use crate::{Habit, HabitStatus, constants::*, error::ErrorCode};

#[derive(Accounts)]
pub struct PauseHabit<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [HABIT_SEED, owner.key().as_ref(), &habit.id.to_le_bytes()],
        bump,
        has_one = owner @ ErrorCode::UnauthorizedHabit,
        constraint = habit.status == HabitStatus::Active @ ErrorCode::HabitNotPauseable
    )]
    pub habit: Account<'info, Habit>,
}

pub fn handle_pause_habit(ctx: Context<PauseHabit>) -> Result<()> {
    ctx.accounts.habit.status = HabitStatus::Paused;
    Ok(())
}
