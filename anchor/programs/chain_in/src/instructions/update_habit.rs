use anchor_lang::prelude::*;

use crate::{Habit, constants::*, error::ErrorCode};

#[derive(Accounts)]
pub struct UpdateHabit<'info> {
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [HABIT_SEED, owner.key().as_ref(), &habit.id.to_le_bytes()],
        bump,
        has_one = owner @ ErrorCode::UnauthorizedHabit
    )]
    pub habit: Account<'info, Habit>,
}

pub fn handle_update_habit(
    ctx: Context<UpdateHabit>,
    name: String,
    description: String,
) -> Result<()> {
    require!(
        name.len() <= MAX_HABIT_NAME_LEN,
        ErrorCode::HabitNameTooLong
    );
    require!(
        description.len() <= MAX_HABIT_DESCRIPTION_LEN,
        ErrorCode::HabitDescriptionTooLong
    );

    let habit = &mut ctx.accounts.habit;
    habit.name = name;
    habit.description = description;

    Ok(())
}
