use anchor_lang::prelude::*;

use crate::{Habit, HabitStatus, constants::*, error::ErrorCode};

#[derive(Accounts)]
pub struct ResumeHabit<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [HABIT_SEED, owner.key().as_ref(), &habit.id.to_le_bytes()],
        bump,
        has_one = owner @ ErrorCode::UnauthorizedHabit,
        constraint = habit.status == HabitStatus::Paused @ ErrorCode::HabitNotResumable
    )]
    pub habit: Account<'info, Habit>,
}

pub fn handle_resume_habit(ctx: Context<ResumeHabit>) -> Result<()> {
    ctx.accounts.habit.status = HabitStatus::Active;
    Ok(())
}
