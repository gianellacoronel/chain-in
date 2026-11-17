use anchor_lang::prelude::*;

use crate::{
    DailyCheckIn, Habit, HabitStatus,
    constants::*,
    error::ErrorCode,
};

#[derive(Accounts)]
#[instruction(day: i64)]
pub struct CheckIn<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(
        mut,
        seeds = [HABIT_SEED, owner.key().as_ref(), &habit.id.to_le_bytes()],
        bump,
        has_one = owner @ ErrorCode::UnauthorizedHabit,
        constraint = habit.status == HabitStatus::Active @ ErrorCode::HabitNotActive
    )]
    pub habit: Account<'info, Habit>,
    #[account(
        init,
        payer = owner,
        space = 8 + DailyCheckIn::INIT_SPACE,
        seeds = [CHECKIN_SEED, habit.key().as_ref(), &day.to_le_bytes()],
        bump
    )]
    pub daily_check_in: Account<'info, DailyCheckIn>,
    pub system_program: Program<'info, System>,
}

pub fn handle_check_in(ctx: Context<CheckIn>, day: i64) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let expected_day = now
        .checked_div(SECONDS_PER_DAY)
        .ok_or_else(|| error!(ErrorCode::InvalidCheckInDay))?;
    require_eq!(day, expected_day, ErrorCode::InvalidCheckInDay);

    let habit = &mut ctx.accounts.habit;
    let new_streak = match habit.last_check_in_day {
        None => 1u32,
        Some(last_day) => {
            let previous_day = day
                .checked_sub(1)
                .ok_or_else(|| error!(ErrorCode::InvalidCheckInDay))?;
            if last_day == previous_day {
                habit
                    .current_streak
                    .checked_add(1)
                    .ok_or_else(|| error!(ErrorCode::StreakOverflow))?
            } else {
                1
            }
        }
    };

    habit.current_streak = new_streak;
    if new_streak > habit.longest_streak {
        habit.longest_streak = new_streak;
    }
    habit.last_check_in_day = Some(day);

    let daily_check_in = &mut ctx.accounts.daily_check_in;
    daily_check_in.owner = ctx.accounts.owner.key();
    daily_check_in.habit = habit.key();
    daily_check_in.habit_id = habit.id;
    daily_check_in.habit_name_snapshot = habit.name.clone();
    daily_check_in.day = day;
    daily_check_in.checked_at = now;

    Ok(())
}
