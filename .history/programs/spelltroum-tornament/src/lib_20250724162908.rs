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
        let signer = &ctx.accounts.owner;

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

    pub fn withdraw<'info>(ctx: Context<'_, '_, '_, 'info, Withdraw<'info>>) -> Result<()> {
        let player = &mut ctx.accounts.player_balance;
        let signer = &ctx.accounts.owner;

        let amount = player.amount;
        require!(amount > 0, CustomError::NothingToWithdraw);
        require!(
            **player.to_account_info().lamports.borrow() >= amount,
            CustomError::InsufficientBalance
        );

        **player.to_account_info().lamports.borrow_mut() -= amount;
        **signer.to_account_info().lamports.borrow_mut() += amount;

        player.amount = 0;
        Ok(())
    }

    pub fn settle_match<'info>(
        ctx: Context<'info, SettleMatch<'info>>, // ✅ явно указали 'info
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

        // 💸 Списываем entry_fee со всех игроков
        for i in 0..players.len() {
            let balance_info = &ctx.remaining_accounts[i];
            let mut player_balance: Account<'info, PlayerBalance> =
                Account::try_from(balance_info)?;
            require!(
                player_balance.amount >= entry_fee,
                CustomError::InsufficientBalance
            );
            player_balance.amount -= entry_fee;
        }

        // 🏆 Начисляем приз победителям
        for i in 0..winners.len() {
            let balance_info = &ctx.remaining_accounts[players.len() + i];
            let mut player_balance: Account<'info, PlayerBalance> =
                Account::try_from(balance_info)?;
            player_balance.amount += share;
        }

        match_history.settled_matches.push(match_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    #[account(mut, has_one = owner)]
    pub player_balance: Account<'info, PlayerBalance>,
    pub system_program: Program<'info, System>,
}

#[account]
pub struct PlayerBalance {
    pub owner: Pubkey,
    pub amount: u64,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
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
