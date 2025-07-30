use anchor_lang::prelude::*;

declare_id!("DteaY2ai6KLh724t4GGwSGWgQHYHu5N1h3SmkRJcWX8j");

#[program]
pub mod spelltroum_tornament {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
