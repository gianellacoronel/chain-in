pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Eai82xNfih16jrTcQktnKaAYWG9hkvUuQ6uRGPtDsWwM");

#[program]
pub mod chain_in {
    use super::*;

    pub fn initialize_profile(ctx: Context<InitializeProfile>) -> Result<()> {
        crate::instructions::initialize::handle_initialize_profile(ctx)
    }

    pub fn create_habit(
        ctx: Context<CreateHabit>,
        name: String,
        description: String,
    ) -> Result<()> {
        crate::instructions::create_habit::handle_create_habit(ctx, name, description)
    }

    pub fn update_habit(
        ctx: Context<UpdateHabit>,
        name: String,
        description: String,
    ) -> Result<()> {
        crate::instructions::update_habit::handle_update_habit(ctx, name, description)
    }

    pub fn check_in(ctx: Context<CheckIn>, day: i64) -> Result<()> {
        crate::instructions::check_in::handle_check_in(ctx, day)
    }

    pub fn pause_habit(ctx: Context<PauseHabit>) -> Result<()> {
        crate::instructions::pause_habit::handle_pause_habit(ctx)
    }

    pub fn resume_habit(ctx: Context<ResumeHabit>) -> Result<()> {
        crate::instructions::resume_habit::handle_resume_habit(ctx)
    }

    pub fn archive_habit(ctx: Context<ArchiveHabit>) -> Result<()> {
        crate::instructions::archive_habit::handle_archive_habit(ctx)
    }

    pub fn close_habit(ctx: Context<CloseHabit>) -> Result<()> {
        crate::instructions::close_habit::handle_close_habit(ctx)
    }
}
