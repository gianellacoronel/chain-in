use anchor_lang::prelude::*;

use crate::{
    Habit, HabitStatus, Profile,
    constants::*,
    error::ErrorCode,
};

#[derive(Accounts)]
pub struct CreateHabit<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [PROFILE_SEED, owner.key().as_ref()],
        bump,
        constraint = profile.authority == owner.key() @ ErrorCode::UnauthorizedProfile
    )]
    pub profile: Account<'info, Profile>,
    #[account(
        init,
        payer = owner,
        space = 8 + Habit::INIT_SPACE,
        seeds = [HABIT_SEED, owner.key().as_ref(), &profile.next_habit_id.to_le_bytes()],
        bump
    )]
    pub habit: Account<'info, Habit>,
    pub system_program: Program<'info, System>,
}

pub fn handle_create_habit(
    ctx: Context<CreateHabit>,
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

    let profile = &mut ctx.accounts.profile;
    let habit_id = profile.next_habit_id;

    let habit = &mut ctx.accounts.habit;
    habit.owner = ctx.accounts.owner.key();
    habit.id = habit_id;
    habit.name = name;
    habit.description = description;
    habit.status = HabitStatus::Active;
    habit.current_streak = 0;
    habit.longest_streak = 0;
    habit.last_check_in_day = None;
    habit.created_at = Clock::get()?.unix_timestamp;

    profile.next_habit_id = habit_id
        .checked_add(1)
        .ok_or(ErrorCode::HabitIdOverflow)?;

    Ok(())
}
