use anchor_lang::prelude::*;

use crate::{Habit, HabitStatus, constants::*, error::ErrorCode};

#[derive(Accounts)]
pub struct ArchiveHabit<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [HABIT_SEED, owner.key().as_ref(), &habit.id.to_le_bytes()],
        bump,
        has_one = owner @ ErrorCode::UnauthorizedHabit,
        constraint = habit.status == HabitStatus::Active
            || habit.status == HabitStatus::Paused
            @ ErrorCode::HabitNotArchivable
    )]
    pub habit: Account<'info, Habit>,
}

pub fn handle_archive_habit(ctx: Context<ArchiveHabit>) -> Result<()> {
    ctx.accounts.habit.status = HabitStatus::Archived;
    Ok(())
}
