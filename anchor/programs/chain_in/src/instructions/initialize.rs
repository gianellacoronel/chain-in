use anchor_lang::prelude::*;

use crate::{Profile, constants::*, state::Counter};

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

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        space = 8 + Counter::INIT_SPACE,
        seeds = [COUNTER_SEED],
        bump
    )]
    pub counter: Account<'info, Counter>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(ctx: Context<Initialize>) -> Result<()> {
    ctx.accounts.counter.count = 0;
    ctx.accounts.counter.authority = ctx.accounts.payer.key();

    let cpi_accounts = anchor_lang::system_program::Transfer {
        from: ctx.accounts.payer.to_account_info(),
        to: ctx.accounts.counter.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(anchor_lang::system_program::ID, cpi_accounts);
    anchor_lang::system_program::transfer(cpi_ctx, HELLO_WORLD_LAMPORTS)?;

    msg!("Hello, world! Counter initialized");
    Ok(())
}
