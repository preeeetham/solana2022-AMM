use anchor_lang::prelude::*;

declare_id!("BroadwayHooK11111111111111111111111111111111");

#[program]
pub mod safe_transfer_hook {
    use super::*;

    /// Execute transfer hook - the main function called during Token-2022 transfers
    pub fn execute(ctx: Context<Execute>, amount: u64) -> Result<()> {
        msg!("Safe Transfer Hook: Executing transfer of {} tokens", amount);
        
        // This is a "safe" hook that performs basic validation and logging
        // In a real-world scenario, this could implement:
        // - Wallet whitelisting
        // - Transaction limits
        // - Time-based restrictions
        // - Compliance checks
        
        let transfer_hook_accounts = &ctx.accounts;
        
        // Log transfer details for monitoring
        msg!(
            "Transfer: {} -> {} (amount: {})", 
            transfer_hook_accounts.source_token.key(),
            transfer_hook_accounts.destination_token.key(),
            amount
        );
        
        // Perform basic validation
        require!(
            amount > 0,
            SafeTransferHookError::InvalidAmount
        );
        
        // Example: Simple rate limiting check (in production, this would use a PDA to store state)
        // For now, we just log and approve all transfers
        msg!("Transfer approved by Safe Transfer Hook");
        
        Ok(())
    }

    /// Initialize extra account metas - called when setting up the transfer hook
    pub fn initialize_extra_account_metas(
        _ctx: Context<InitializeExtraAccountMetas>,
    ) -> Result<()> {
        msg!("Initializing extra account metas for Safe Transfer Hook");
        
        // This hook doesn't require any extra accounts beyond the standard ones
        // In more complex hooks, you might require additional accounts like:
        // - Authority accounts
        // - State accounts
        // - Oracle accounts
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Execute<'info> {
    /// The source token account
    /// CHECK: This is validated by the Token-2022 program
    pub source_token: UncheckedAccount<'info>,
    
    /// The mint account
    /// CHECK: This is validated by the Token-2022 program
    pub mint: UncheckedAccount<'info>,
    
    /// The destination token account
    /// CHECK: This is validated by the Token-2022 program
    pub destination_token: UncheckedAccount<'info>,
    
    /// The token account owner/delegate
    /// CHECK: This is validated by the Token-2022 program
    pub owner: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct InitializeExtraAccountMetas<'info> {
    /// The extra account metas account
    /// CHECK: This account is used by the transfer hook interface
    #[account(mut)]
    pub extra_account_metas: UncheckedAccount<'info>,
    
    /// The mint account
    /// CHECK: This is validated by the Token-2022 program
    pub mint: UncheckedAccount<'info>,
    
    /// The authority for the extra account metas
    pub authority: Signer<'info>,
    
    /// System program for account creation
    pub system_program: Program<'info, System>,
}

#[error_code]
pub enum SafeTransferHookError {
    #[msg("Invalid transfer amount")]
    InvalidAmount,
    #[msg("Transfer not authorized")]
    NotAuthorized,
    #[msg("Rate limit exceeded")]
    RateLimitExceeded,
}

// Security features that this hook demonstrates:
// 1. Input validation (amount > 0)
// 2. Transfer logging for audit trails
// 3. Extensible architecture for additional security measures
// 4. Clear error handling and messaging
//
// This hook is designed to be a "known safe" program that can be whitelisted
// in the AMM without introducing security risks. It serves as a template
// for implementing more sophisticated transfer restrictions while maintaining
// the security guarantees of the whitelist system.