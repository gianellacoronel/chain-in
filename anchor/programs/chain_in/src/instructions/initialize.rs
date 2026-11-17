use anchor_lang::prelude::*;

use crate::{Profile, constants::*};

#[derive(Accounts)]
pub struct InitializeProfile<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init,
        payer = authority,
        space = 8 + Profile::INIT_SPACE,
        seeds = [PROFILE_SEED, authority.key().as_ref()],
        bump
    )]
    pub profile: Account<'info, Profile>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_profile(ctx: Context<InitializeProfile>) -> Result<()> {
    let profile = &mut ctx.accounts.profile;
    profile.authority = ctx.accounts.authority.key();
    profile.next_habit_id = 0;
    profile.created_at = Clock::get()?.unix_timestamp;
    Ok(())
}
