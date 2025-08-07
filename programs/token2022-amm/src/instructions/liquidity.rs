use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint, transfer, mint_to, MintTo, Transfer};
use anchor_spl::token_2022::{Token2022, transfer_checked, TransferChecked};
use crate::state::{AmmPool, TransferHookWhitelist};
use crate::error::AmmError;

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(mut)]
    pub pool: Account<'info, AmmPool>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// User's token A account
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    
    /// User's token B account
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,
    
    /// User's LP token account
    #[account(mut)]
    pub user_lp_token: Account<'info, TokenAccount>,
    
    /// Pool's token A vault
    #[account(mut)]
    pub pool_token_a_vault: Account<'info, TokenAccount>,
    
    /// Pool's token B vault
    #[account(mut)]
    pub pool_token_b_vault: Account<'info, TokenAccount>,
    
    /// Pool's LP token mint
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    
    /// Token A mint
    pub token_a_mint: Account<'info, Mint>,
    
    /// Token B mint
    pub token_b_mint: Account<'info, Mint>,
    
    /// Transfer Hook Whitelist for validation
    pub whitelist: Account<'info, TransferHookWhitelist>,
    
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
}

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(mut)]
    pub pool: Account<'info, AmmPool>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// User's token A account
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    
    /// User's token B account
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,
    
    /// User's LP token account
    #[account(mut)]
    pub user_lp_token: Account<'info, TokenAccount>,
    
    /// Pool's token A vault
    #[account(mut)]
    pub pool_token_a_vault: Account<'info, TokenAccount>,
    
    /// Pool's token B vault
    #[account(mut)]
    pub pool_token_b_vault: Account<'info, TokenAccount>,
    
    /// Pool's LP token mint
    #[account(mut)]
    pub lp_mint: Account<'info, Mint>,
    
    /// Token A mint
    pub token_a_mint: Account<'info, Mint>,
    
    /// Token B mint
    pub token_b_mint: Account<'info, Mint>,
    
    /// Transfer Hook Whitelist for validation
    pub whitelist: Account<'info, TransferHookWhitelist>,
    
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
}

pub fn add_liquidity(
    ctx: Context<AddLiquidity>,
    amount_a: u64,
    amount_b: u64,
    min_lp_tokens: u64,
) -> Result<()> {
    let user = &ctx.accounts.user;
    let pool_account_info = ctx.accounts.pool.to_account_info();
    
    // Get pool data before mutable borrow
    let pool = &mut ctx.accounts.pool;
    let lp_tokens_to_mint = pool.calculate_lp_tokens_for_liquidity(amount_a, amount_b)?;
    let pool_bump = pool.bump;
    
    // Check minimum LP tokens
    require!(
        lp_tokens_to_mint >= min_lp_tokens,
        AmmError::InsufficientLPTokens
    );
    
    // Transfer token A from user to pool using Token-2022
    let transfer_a_ctx = CpiContext::new(
        ctx.accounts.token_2022_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.user_token_a.to_account_info(),
            mint: ctx.accounts.token_a_mint.to_account_info(),
            to: ctx.accounts.pool_token_a_vault.to_account_info(),
            authority: user.to_account_info(),
        },
    );
    transfer_checked(transfer_a_ctx, amount_a, ctx.accounts.token_a_mint.decimals)?;
    
    // Transfer token B from user to pool using Token-2022
    let transfer_b_ctx = CpiContext::new(
        ctx.accounts.token_2022_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.user_token_b.to_account_info(),
            mint: ctx.accounts.token_b_mint.to_account_info(),
            to: ctx.accounts.pool_token_b_vault.to_account_info(),
            authority: user.to_account_info(),
        },
    );
    transfer_checked(transfer_b_ctx, amount_b, ctx.accounts.token_b_mint.decimals)?;
    
    // Mint LP tokens to user
    let pool_seeds: &[&[u8]] = &[b"pool", &[pool_bump]];
    let signer_seeds = &[pool_seeds];
    
    let mint_lp_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.lp_mint.to_account_info(),
            to: ctx.accounts.user_lp_token.to_account_info(),
            authority: pool_account_info.clone(),
        },
        signer_seeds,
    );
    mint_to(mint_lp_ctx, lp_tokens_to_mint)?;
    
    // Update pool state
    pool.add_liquidity(amount_a, amount_b, lp_tokens_to_mint)?;
    
    msg!("Liquidity added successfully");
    msg!("Token A: {}", amount_a);
    msg!("Token B: {}", amount_b);
    msg!("LP Tokens: {}", lp_tokens_to_mint);
    
    Ok(())
}

pub fn remove_liquidity(
    ctx: Context<RemoveLiquidity>,
    lp_tokens_to_burn: u64,
    min_token_a: u64,
    min_token_b: u64,
) -> Result<()> {
    let user = &ctx.accounts.user;
    let pool_account_info = ctx.accounts.pool.to_account_info();
    
    // Get pool data before mutable borrow
    let pool = &mut ctx.accounts.pool;
    let (token_a_amount, token_b_amount) = pool.calculate_tokens_for_lp_burn(lp_tokens_to_burn)?;
    let pool_bump = pool.bump;
    
    // Check minimum amounts
    require!(
        token_a_amount >= min_token_a,
        AmmError::InsufficientTokenA
    );
    require!(
        token_b_amount >= min_token_b,
        AmmError::InsufficientTokenB
    );
    
    // Burn LP tokens from user
    let burn_lp_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.user_lp_token.to_account_info(),
            to: ctx.accounts.pool_token_a_vault.to_account_info(), // Dummy transfer to burn
            authority: user.to_account_info(),
        },
    );
    transfer(burn_lp_ctx, lp_tokens_to_burn)?;
    
    // Transfer token A from pool to user using Token-2022
    let pool_seeds: &[&[u8]] = &[b"pool", &[pool_bump]];
    let signer_seeds = &[pool_seeds];
    
    let transfer_a_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_2022_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.pool_token_a_vault.to_account_info(),
            mint: ctx.accounts.token_a_mint.to_account_info(),
            to: ctx.accounts.user_token_a.to_account_info(),
            authority: pool_account_info.clone(),
        },
        signer_seeds,
    );
    transfer_checked(transfer_a_ctx, token_a_amount, ctx.accounts.token_a_mint.decimals)?;
    
    // Transfer token B from pool to user using Token-2022
    let transfer_b_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_2022_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.pool_token_b_vault.to_account_info(),
            mint: ctx.accounts.token_b_mint.to_account_info(),
            to: ctx.accounts.user_token_b.to_account_info(),
            authority: pool_account_info.clone(),
        },
        signer_seeds,
    );
    transfer_checked(transfer_b_ctx, token_b_amount, ctx.accounts.token_b_mint.decimals)?;
    
    // Update pool state
    pool.remove_liquidity(token_a_amount, token_b_amount, lp_tokens_to_burn)?;
    
    msg!("Liquidity removed successfully");
    msg!("LP Tokens burned: {}", lp_tokens_to_burn);
    msg!("Token A returned: {}", token_a_amount);
    msg!("Token B returned: {}", token_b_amount);
    
    Ok(())
} 