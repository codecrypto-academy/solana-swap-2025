use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, transfer, Transfer, TransferChecked};

declare_id!("C3RC2picGe1SRyYNny7bVWCtVc5qizqd7XBMErby2J6V");

#[program]
pub mod solana_swap_2025 {
    use super::*;

    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        price: u64,
        decimals_a: u8,
        decimals_b: u8,
        bump: u8,
    ) -> Result<()> {
        let market = &mut ctx.accounts.market;
        market.authority = ctx.accounts.authority.key();
        market.token_mint_a = ctx.accounts.token_mint_a.key();
        market.token_mint_b = ctx.accounts.token_mint_b.key();
        market.price = price;
        market.decimals_a = decimals_a;
        market.decimals_b = decimals_b;
        market.bump = bump;
        Ok(())
    }

    pub fn set_price(ctx: Context<SetPrice>, price: u64) -> Result<()> {
        let market = &mut ctx.accounts.market;
        market.price = price;
        Ok(())
    }

    pub fn add_liquidity(ctx: Context<AddLiquidity>, amount_a: u64, amount_b: u64) -> Result<()> {
        let market = &mut ctx.accounts.market;
        let vault_a = &mut ctx.accounts.vault_a;
        let vault_b = &mut ctx.accounts.vault_b;
        if amount_a > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.autority_token_a.to_account_info(),
                to: ctx.accounts.vault_a.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            transfer(CpiContext::new(cpi_program, cpi_accounts), amount_a)?;
        }
        if amount_b > 0 {
            let cpi_accounts = Transfer {
                from: ctx.accounts.autority_token_b.to_account_info(),
                to: ctx.accounts.vault_b.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            transfer(CpiContext::new(cpi_program, cpi_accounts), amount_b)?;
        }
        Ok(())
    }

    pub fn swap(ctx: Context<Swap>, amount: u64, a_to_b: bool) -> Result<()> {
        let market = &mut ctx.accounts.market;
        if (a_to_b) {
            let cpi_accounts = Transfer {
                from: ctx.accounts.user_token_a.to_account_info(),
                to: ctx.accounts.vault_a.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.to_account_info();
            transfer(CpiContext::new(cpi_program, cpi_accounts), amount)?;

            let cpi_account2 = Transfer{
                from: ctx.accounts.vault_b.to_account_info(),
                to: ctx.accounts.user_token_b.to_account_info(),
                authority: market.to_account_info(),
            };
           

            let amount_b: u64 = amount * market.price;
            let cpi_program2 = ctx.accounts.token_program.to_account_info();
            let signer_seeds: &[&[&[u8]]] = &[&[
                b"market",
                market.token_mint_a.as_ref(),
                market.token_mint_b.as_ref(),
                &[market.bump], 
            ]];
            transfer(CpiContext::new_with_signer(cpi_program2, 
                cpi_account2, signer_seeds), amount_b)?;
        } else {
            
        }
        Ok(())
    }   
}

#[derive(Accounts)]
pub struct SetPrice<'info> {
    pub token_mint_a: Account<'info, Mint>,
    pub token_mint_b: Account<'info, Mint>,
    #[account(mut,
        seeds = [b"market".as_ref(), token_mint_a.key().as_ref(), token_mint_b.key().as_ref()],
        bump,
    )]
    pub market: Account<'info, MarketAccount>,
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeMarket<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + MarketAccount::INIT_SPACE,
        seeds = [b"market".as_ref(), token_mint_a.key().as_ref(), token_mint_b.key().as_ref()],
        bump
    )]
    pub market: Account<'info, MarketAccount>,

    pub token_mint_a: Account<'info, Mint>,
    pub token_mint_b: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        token::mint = token_mint_a,
        token::authority = market,
        seeds = [b"vault_a".as_ref(), market.key().as_ref()],
        bump,
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        token::mint = token_mint_b,
        token::authority = market,
        seeds = [b"vault_b".as_ref(), market.key().as_ref()],
        bump,
    )]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}


#[derive(Accounts)]
pub struct Swap<'info> {
    pub token_mint_a: Account<'info, Mint>,
    pub token_mint_b: Account<'info, Mint>,
    #[account(mut,
        seeds = [b"market".as_ref(), token_mint_a.key().as_ref(), token_mint_b.key().as_ref()],
        bump
    )]
    pub market: Account<'info, MarketAccount>,

    #[account(mut,
        seeds = [b"vault_a".as_ref(), market.key().as_ref()],
        bump,
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(mut,
        seeds = [b"vault_b".as_ref(), market.key().as_ref()],
        bump,
    )]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user : Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    pub token_mint_a: Account<'info, Mint>,
    pub token_mint_b: Account<'info, Mint>,
    #[account(mut,
        seeds = [b"market".as_ref(), token_mint_a.key().as_ref(), token_mint_b.key().as_ref()],
        bump
    )]
    pub market: Account<'info, MarketAccount>,

    #[account(mut,
        seeds = [b"vault_a".as_ref(), market.key().as_ref()],
        bump,
    )]
    pub vault_a: Account<'info, TokenAccount>,

    #[account(mut,
        seeds = [b"vault_b".as_ref(), market.key().as_ref()],
        bump,
    )]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub autority_token_a: Account<'info, TokenAccount>,
    #[account(mut)]
    pub autority_token_b: Account<'info, TokenAccount>,

    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>
}
#[account]
#[derive(InitSpace)]
pub struct MarketAccount {
    pub authority: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub price: u64,
    pub decimals_a: u8,
    pub decimals_b: u8,
    pub bump: u8,
}
