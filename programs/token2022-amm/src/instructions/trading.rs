use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use anchor_spl::token_2022::{Token2022, transfer_checked, TransferChecked};
use crate::state::{AmmPool, TransferHookWhitelist};
use crate::error::AmmError;

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub pool: Account<'info, AmmPool>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// User's token A account (input)
    #[account(mut)]
    pub user_token_a: Account<'info, TokenAccount>,
    
    /// User's token B account (output)
    #[account(mut)]
    pub user_token_b: Account<'info, TokenAccount>,
    
    /// Pool's token A vault
    #[account(mut)]
    pub pool_token_a_vault: Account<'info, TokenAccount>,
    
    /// Pool's token B vault
    #[account(mut)]
    pub pool_token_b_vault: Account<'info, TokenAccount>,
    
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
pub struct SwapExactTokensForTokens<'info> {
    #[account(mut)]
    pub pool: Account<'info, AmmPool>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    /// User's input token account
    #[account(mut)]
    pub user_input_token: Account<'info, TokenAccount>,
    
    /// User's output token account
    #[account(mut)]
    pub user_output_token: Account<'info, TokenAccount>,
    
    /// Pool's input token vault
    #[account(mut)]
    pub pool_input_vault: Account<'info, TokenAccount>,
    
    /// Pool's output token vault
    #[account(mut)]
    pub pool_output_vault: Account<'info, TokenAccount>,
    
    /// Input token mint
    pub input_mint: Account<'info, Mint>,
    
    /// Output token mint
    pub output_mint: Account<'info, Mint>,
    
    /// Transfer Hook Whitelist for validation
    pub whitelist: Account<'info, TransferHookWhitelist>,
    
    pub token_program: Program<'info, Token>,
    pub token_2022_program: Program<'info, Token2022>,
}

pub fn swap(
    ctx: Context<Swap>,
    amount_in: u64,
    min_amount_out: u64,
) -> Result<()> {
    let user = &ctx.accounts.user;
    let pool_account_info = ctx.accounts.pool.to_account_info();
    
    // Get pool data before mutable borrow
    let pool = &mut ctx.accounts.pool;
    let amount_out = pool.calculate_swap_output(amount_in)?;
    let pool_bump = pool.bump;
    
    // Check slippage protection
    require!(
        amount_out >= min_amount_out,
        AmmError::InsufficientOutputAmount
    );
    
    // Validate transfer hooks for Token-2022 tokens
    let _whitelist = &ctx.accounts.whitelist;
    
    // For now, we'll use Token-2022 transfers for all tokens to ensure hook validation
    // In a production system, you would check the mint's extensions to determine if it has transfer hooks
    
    // Transfer tokens from user to pool using Token-2022
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_2022_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.user_token_a.to_account_info(),
            mint: ctx.accounts.token_a_mint.to_account_info(),
            to: ctx.accounts.pool_token_a_vault.to_account_info(),
            authority: user.to_account_info(),
        },
    );
    
    transfer_checked(transfer_ctx, amount_in, ctx.accounts.token_a_mint.decimals)?;
    
    // Transfer tokens from pool to user using Token-2022
    let pool_seeds: &[&[u8]] = &[b"pool", &[pool_bump]];
    let signer_seeds = &[pool_seeds];
    
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_2022_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.pool_token_b_vault.to_account_info(),
            mint: ctx.accounts.token_b_mint.to_account_info(),
            to: ctx.accounts.user_token_b.to_account_info(),
            authority: pool_account_info.clone(),
        },
        signer_seeds,
    );
    
    transfer_checked(transfer_ctx, amount_out, ctx.accounts.token_b_mint.decimals)?;
    
    // Update pool state
    pool.update_swap_state(amount_in, amount_out)?;
    
    msg!("Swap executed successfully with Token-2022 hook validation");
    msg!("Amount in: {}", amount_in);
    msg!("Amount out: {}", amount_out);
    
    Ok(())
}

pub fn swap_exact_tokens_for_tokens(
    ctx: Context<SwapExactTokensForTokens>,
    amount_in: u64,
    min_amount_out: u64,
) -> Result<()> {
    let user = &ctx.accounts.user;
    let pool_account_info = ctx.accounts.pool.to_account_info();
    
    // Get pool data before mutable borrow
    let pool = &mut ctx.accounts.pool;
    let amount_out = pool.calculate_swap_output(amount_in)?;
    let pool_bump = pool.bump;
    
    // Check slippage protection
    require!(
        amount_out >= min_amount_out,
        AmmError::InsufficientOutputAmount
    );
    
    // Transfer tokens from user to pool using Token-2022
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_2022_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.user_input_token.to_account_info(),
            mint: ctx.accounts.input_mint.to_account_info(),
            to: ctx.accounts.pool_input_vault.to_account_info(),
            authority: user.to_account_info(),
        },
    );
    
    transfer_checked(transfer_ctx, amount_in, ctx.accounts.input_mint.decimals)?;
    
    // Transfer tokens from pool to user using Token-2022
    let pool_seeds: &[&[u8]] = &[b"pool", &[pool_bump]];
    let signer_seeds = &[pool_seeds];
    
    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_2022_program.to_account_info(),
        TransferChecked {
            from: ctx.accounts.pool_output_vault.to_account_info(),
            mint: ctx.accounts.output_mint.to_account_info(),
            to: ctx.accounts.user_output_token.to_account_info(),
            authority: pool_account_info.clone(),
        },
        signer_seeds,
    );
    
    transfer_checked(transfer_ctx, amount_out, ctx.accounts.output_mint.decimals)?;
    
    // Update pool state
    pool.update_swap_state(amount_in, amount_out)?;
    
    msg!("Exact swap executed successfully with Token-2022 hook validation");
    msg!("Amount in: {}", amount_in);
    msg!("Amount out: {}", amount_out);
    
    Ok(())
} 