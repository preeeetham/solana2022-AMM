use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::AmmPool;
use crate::error::AmmError;

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<AmmPool>()
    )]
    pub pool: Account<'info, AmmPool>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// Token A mint (e.g., SOL)
    pub token_a_mint: Account<'info, Mint>,
    
    /// Token B mint (e.g., Token-2022)
    pub token_b_mint: Account<'info, Mint>,
    
    /// Pool's token A vault
    #[account(
        init,
        payer = authority,
        token::mint = token_a_mint,
        token::authority = pool,
    )]
    pub token_a_vault: Account<'info, TokenAccount>,
    
    /// Pool's token B vault
    #[account(
        init,
        payer = authority,
        token::mint = token_b_mint,
        token::authority = pool,
    )]
    pub token_b_vault: Account<'info, TokenAccount>,
    
    /// Pool's LP token mint
    #[account(
        init,
        payer = authority,
        mint::decimals = 6,
        mint::authority = pool,
    )]
    pub lp_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UpdatePoolConfig<'info> {
    #[account(
        mut,
        has_one = authority @ AmmError::InvalidPoolAuthority
    )]
    pub pool: Account<'info, AmmPool>,
    
    pub authority: Signer<'info>,
}

pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let authority = &ctx.accounts.authority;
    
    // Initialize pool with basic configuration
    pool.initialize(
        authority.key(),
        ctx.accounts.token_a_mint.key(),
        ctx.accounts.token_b_mint.key(),
        ctx.accounts.token_a_vault.key(),
        ctx.accounts.token_b_vault.key(),
        ctx.accounts.lp_mint.key(),
    )?;
    
    msg!("AMM Pool initialized successfully");
    msg!("Token A: {}", ctx.accounts.token_a_mint.key());
    msg!("Token B: {}", ctx.accounts.token_b_mint.key());
    msg!("LP Mint: {}", ctx.accounts.lp_mint.key());
    
    Ok(())
}

pub fn update_pool_config(
    ctx: Context<UpdatePoolConfig>,
    fee_rate: u64,
    min_liquidity: u64,
) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    
    pool.update_config(fee_rate, min_liquidity)?;
    
    msg!("Pool configuration updated");
    msg!("Fee rate: {}", fee_rate);
    msg!("Min liquidity: {}", min_liquidity);
    
    Ok(())
} 