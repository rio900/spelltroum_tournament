use anchor_lang::prelude::*;
use std::collections::BTreeMap;

declare_id!("DteaY2ai6KLh724t4GGwSGWgQHYHu5N1h3SmkRJcWX8j");

#[program]
pub mod spelltroum_match {
    use super::*;

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let player = &mut ctx.accounts.player_balance;
        let signer = &ctx.accounts.signer;

        require!(amount > 0, CustomError::InvalidAmount);

        **player.to_account_info().lamports.borrow_mut() += amount;
        **signer.to_account_info().lamports.borrow_mut() -= amount;

        player.amount += amount;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        let player = &mut ctx.accounts.player_balance;
        let signer = &ctx.accounts.signer;

        let amount = player.amount;
        require!(amount > 0, CustomError::NothingToWithdraw);

        **signer.to_account_info().lamports.borrow_mut() += amount;
        **player.to_account_info().lamports.borrow_mut() -= amount;

        player.amount = 0;
        Ok(())
    }

    pub fn settle_match(
        ctx: Context<SettleMatch>,
        match_id: String,
        players: Vec<Pubkey>,
        winners: Vec<Pubkey>,
        entry_fee: u64,
    ) -> Result<()> {
        let match_history = &mut ctx.accounts.match_history;
        require!(
            !match_history.settled_matches.contains(&match_id),
            CustomError::MatchAlreadySettled
        );

        let prize_pool = entry_fee
            .checked_mul(players.len() as u64)
            .ok_or(CustomError::MathOverflow)?;
        let share = prize_pool / winners.len() as u64;

        // Deduct entry fee from players
        for i in 0..players.len() {
            let balance = &mut ctx.remaining_accounts[i];
            let mut player_balance: Account<PlayerBalance> = Account::try_from(balance)?;
            require!(
                player_balance.amount >= entry_fee,
                CustomError::InsufficientBalance
            );
            player_balance.amount -= entry_fee;
        }

        // Add share to winners
        for i in 0..winners.len() {
            let balance = &mut ctx.remaining_accounts[players.len() + i];
            let mut player_balance: Account<PlayerBalance> = Account::try_from(balance)?;
            player_balance.amount += share;
        }

        match_history.settled_matches.push(match_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, has_one = owner)]
    pub player_balance: Account<'info, PlayerBalance>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut, has_one = owner)]
    pub player_balance: Account<'info, PlayerBalance>,
}

#[derive(Accounts)]
pub struct SettleMatch<'info> {
    #[account(mut, has_one = authority)]
    pub match_history: Account<'info, MatchHistory>,
    pub authority: Signer<'info>,
}

#[account]
pub struct PlayerBalance {
    pub owner: Pubkey,
    pub amount: u64,
}

#[account]
pub struct MatchHistory {
    pub authority: Pubkey,
    pub settled_matches: Vec<String>,
}

#[error_code]
pub enum CustomError {
    #[msg("Insufficient balance to perform operation")]
    InsufficientBalance,
    #[msg("This match has already been settled")]
    MatchAlreadySettled,
    #[msg("Nothing to withdraw")]
    NothingToWithdraw,
    #[msg("Math overflow error")]
    MathOverflow,
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
}
