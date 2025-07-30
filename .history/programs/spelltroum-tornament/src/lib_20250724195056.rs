use anchor_lang::prelude::*;
use array_init::array_init;

declare_id!("DteaY2ai6KLh724t4GGwSGWgQHYHu5N1h3SmkRJcWX8j");

#[program]
pub mod spelltroum_tournament {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let match_history = &mut ctx.accounts.match_history;
        match_history.authority = ctx.accounts.authority.key();
        match_history.settled_matches = array_init(|_| None);
        match_history.current_index = 0;
        msg!("Initialized MatchHistory");
        Ok(())
    }

    pub fn deposit<'info>(
        ctx: Context<'_, '_, '_, 'info, Deposit<'info>>,
        amount: u64,
    ) -> Result<()> {
        let player = &mut ctx.accounts.player_balance;

        require!(amount > 0, CustomError::InvalidAmount);

        player.amount = player
            .amount
            .checked_add(amount)
            .ok_or(CustomError::MathOverflow)?;

        emit!(Deposited {
            user: ctx.accounts.owner.key(),
            amount,
        });

        Ok(())
    }

    pub fn withdraw<'info>(ctx: Context<'_, '_, '_, 'info, Withdraw<'info>>) -> Result<()> {
        let player = &mut ctx.accounts.player_balance;

        let amount = player.amount;
        require!(amount > 0, CustomError::NothingToWithdraw);

        player.amount = 0;

        emit!(Withdrawn {
            user: ctx.accounts.owner.key(),
            amount,
        });

        Ok(())
    }

    pub fn settle_match<'info>(
        ctx: Context<'_, '_, 'info, 'info, SettleMatch<'info>>,
        match_id: String,
        players: Vec<Pubkey>,
        winners: Vec<Pubkey>,
        entry_fee: u64,
    ) -> Result<()> {
        let match_history = &mut ctx.accounts.match_history;

        // Проверка дубликатов
        require!(
            !match_history
                .settled_matches
                .iter()
                .flatten()
                .any(|m| m == &match_id),
            CustomError::MatchAlreadySettled
        );

        let prize_pool = entry_fee
            .checked_mul(players.len() as u64)
            .ok_or(CustomError::MathOverflow)?;
        let share = prize_pool
            .checked_div(winners.len() as u64)
            .ok_or(CustomError::MathOverflow)?;

        // Удержание entry fee
        for (i, player_key) in players.iter().enumerate() {
            let balance_info = &ctx.remaining_accounts[i];
            let mut balance: Account<PlayerBalance> = Account::try_from(balance_info)?;
            require!(
                balance.owner == *player_key,
                CustomError::InvalidPlayerAccount
            );
            require!(
                balance.amount >= entry_fee,
                CustomError::InsufficientBalance
            );
            balance.amount -= entry_fee;
        }

        // Распределение выигрыша
        for (i, winner_key) in winners.iter().enumerate() {
            let balance_info = &ctx.remaining_accounts[players.len() + i];
            let mut balance: Account<PlayerBalance> = Account::try_from(balance_info)?;
            require!(
                balance.owner == *winner_key,
                CustomError::InvalidWinnerAccount
            );
            balance.amount = balance
                .amount
                .checked_add(share)
                .ok_or(CustomError::MathOverflow)?;
        }

        // Сохраняем match_id циклически
        match_history.settled_matches[match_history.current_index as usize] =
            Some(match_id.clone());
        match_history.current_index = (match_history.current_index + 1) % 1000;

        emit!(MatchSettled {
            match_id,
            total_pool: prize_pool,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + MatchHistory::MAX_SIZE,
        seeds = [b"match_history"],
        bump
    )]
    pub match_history: Account<'info, MatchHistory>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"player", owner.key().as_ref()],
        bump,
        has_one = owner
    )]
    pub player_balance: Account<'info, PlayerBalance>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"player", owner.key().as_ref()],
        bump,
        has_one = owner
    )]
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
    pub bump: u8,
}

#[account]
pub struct MatchHistory {
    pub authority: Pubkey,
    pub settled_matches: [Option<String>; 1000],
    pub current_index: u16,
}

impl MatchHistory {
    pub const MAX_SIZE: usize = 32 // authority
        + (4 + 1000 * (4 + 64)) // Vec<String> with up to 64 bytes per string
        + 2; // current_index
}

#[event]
pub struct Deposited {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct Withdrawn {
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct MatchSettled {
    pub match_id: String,
    pub total_pool: u64,
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
    #[msg("Invalid player account")]
    InvalidPlayerAccount,
    #[msg("Invalid winner account")]
    InvalidWinnerAccount,
}
