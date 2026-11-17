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
}
