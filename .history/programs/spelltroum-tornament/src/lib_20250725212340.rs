use anchor_lang::prelude::*;

declare_id!("DteaY2ai6KLh724t4GGwSGWgQHYHu5N1h3SmkRJcWX8j");

#[program]
pub mod spelltroum_tournament {
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
        require!(!winners.is_empty(), CustomError::NoWinnersProvided);

        require!(
            ctx.accounts
                .match_history
                .settled_matches
                .iter()
                .any(|m| m == &match_id)
                == false,
            CustomError::MatchAlreadySettled
        );

        let prize_pool = entry_fee
            .checked_mul(players.len() as u64)
            .ok_or(CustomError::MathOverflow)?;
        let share = prize_pool
            .checked_div(winners.len() as u64)
            .ok_or(CustomError::MathOverflow)?;

        // Deduct entry fees
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

        // Distribute winnings
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

        let match_history = &mut ctx.accounts.match_history;
        // match_history.settled_matches.push(match_id.clone());
        if match_history.settled_matches.len() < 100 {
            match_history.settled_matches.push(match_id.clone());
        } else {
            let index = match_history.current_index as usize % 100;
            match_history.settled_matches[index] = match_id.clone();
        }
        match_history.current_index = (match_history.current_index + 1) % 100;

        emit!(MatchSettled {
            match_id,
            total_pool: prize_pool,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}

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
    pub settled_matches: Vec<String>,
    pub current_index: u16,
}

#[derive(Accounts)]
pub struct InitPlayer<'info> {
    #[account(init, payer = owner, space = 8 + std::mem::size_of::<PlayerBalance>())]
    pub player_balance: Account<'info, PlayerBalance>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitMatchHistory<'info> {
    #[account(init, payer = authority, space = 8 + std::mem::size_of::<MatchHistory>())]
    pub match_history: Account<'info, MatchHistory>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
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
    #[msg("Too many matches in history")]
    TooManyMatches,
    #[msg("No winners provided")]
    NoWinnersProvided,
}
