use anchor_lang::prelude::*;

declare_id!("DteaY2ai6KLh724t4GGwSGWgQHYHu5N1h3SmkRJcWX8j");

#[program]
pub mod spelltroum_tornament {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }

    pub fn deposit<'info>(
        ctx: Context<'_, '_, '_, 'info, Deposit<'info>>,
        amount: u64,
    ) -> Result<()> {
        let player = &mut ctx.accounts.player_balance;
        let signer = &ctx.accounts.signer;

        require!(amount > 0, CustomError::InvalidAmount);
        require!(
            **signer.to_account_info().lamports.borrow() >= amount,
            CustomError::InsufficientBalance
        );

        **signer.to_account_info().lamports.borrow_mut() -= amount;
        **player.to_account_info().lamports.borrow_mut() += amount;

        player.amount += amount;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
