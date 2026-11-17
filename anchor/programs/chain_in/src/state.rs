use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Counter {
    pub count: u64,
    pub authority: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct Profile {
    pub authority: Pubkey,
    pub next_habit_id: u64,
    pub created_at: i64,
}
