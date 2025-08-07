use anchor_lang::prelude::*;
use crate::state::TransferHookWhitelist;
use crate::error::AmmError;

#[derive(Accounts)]
pub struct InitializeWhitelist<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + std::mem::size_of::<TransferHookWhitelist>()
    )]
    pub whitelist: Account<'info, TransferHookWhitelist>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddHookToWhitelist<'info> {
    #[account(
        mut,
        has_one = authority @ AmmError::InvalidWhitelistAuthority
    )]
    pub whitelist: Account<'info, TransferHookWhitelist>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveHookFromWhitelist<'info> {
    #[account(
        mut,
        has_one = authority @ AmmError::InvalidWhitelistAuthority
    )]
    pub whitelist: Account<'info, TransferHookWhitelist>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ValidateTransferHook<'info> {
    pub whitelist: Account<'info, TransferHookWhitelist>,
}

pub fn initialize_whitelist(ctx: Context<InitializeWhitelist>) -> Result<()> {
    let whitelist = &mut ctx.accounts.whitelist;
    whitelist.initialize(ctx.accounts.authority.key())?;
    
    msg!("Transfer Hook whitelist initialized with authority: {}", ctx.accounts.authority.key());
    Ok(())
}

pub fn add_hook_to_whitelist(
    ctx: Context<AddHookToWhitelist>,
    hook_program_id: Pubkey,
) -> Result<()> {
    let whitelist = &mut ctx.accounts.whitelist;
    whitelist.add_hook(hook_program_id)?;
    
    msg!("Added hook to whitelist: {}", hook_program_id);
    Ok(())
}

pub fn remove_hook_from_whitelist(
    ctx: Context<RemoveHookFromWhitelist>,
    hook_program_id: Pubkey,
) -> Result<()> {
    let whitelist = &mut ctx.accounts.whitelist;
    whitelist.remove_hook(&hook_program_id)?;
    
    msg!("Removed hook from whitelist: {}", hook_program_id);
    Ok(())
}

pub fn validate_transfer_hook(
    ctx: Context<ValidateTransferHook>,
    hook_program_id: Pubkey,
) -> Result<bool> {
    let whitelist = &ctx.accounts.whitelist;
    let is_whitelisted = whitelist.is_hook_whitelisted(&hook_program_id);
    
    if !is_whitelisted {
        return Err(AmmError::HookNotWhitelisted.into());
    }
    
    Ok(true)
}