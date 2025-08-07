use anchor_lang::prelude::*;
use crate::state::{TransferHookWhitelist, HookProposal};
use crate::error::AmmError;

#[derive(Accounts)]
pub struct CreateHookProposal<'info> {
    #[account(
        init,
        payer = proposer,
        space = 8 + std::mem::size_of::<HookProposal>()
    )]
    pub proposal: Account<'info, HookProposal>,
    
    #[account(mut)]
    pub proposer: Signer<'info>,
    
    pub whitelist: Account<'info, TransferHookWhitelist>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, HookProposal>,
    
    #[account(mut)]
    pub voter: Signer<'info>,
    
    pub whitelist: Account<'info, TransferHookWhitelist>,
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    #[account(
        mut,
        constraint = proposal.is_executable() @ AmmError::ProposalNotExecutable
    )]
    pub proposal: Account<'info, HookProposal>,
    
    #[account(
        mut,
        has_one = authority @ AmmError::InvalidWhitelistAuthority
    )]
    pub whitelist: Account<'info, TransferHookWhitelist>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CancelProposal<'info> {
    #[account(
        mut,
        has_one = proposer @ AmmError::InvalidProposalProposer
    )]
    pub proposal: Account<'info, HookProposal>,
    
    pub proposer: Signer<'info>,
}

pub fn create_hook_proposal(
    ctx: Context<CreateHookProposal>,
    hook_program_id: Pubkey,
    description: String,
    audit_report_url: String,
    proposer_stake: u64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let proposer = &ctx.accounts.proposer;
    
    proposal.initialize(
        proposer.key(),
        hook_program_id,
        description.clone(),
        audit_report_url.clone(),
        proposer_stake,
        Clock::get()?.unix_timestamp,
    )?;
    
    msg!("Hook proposal created: {}", hook_program_id);
    msg!("Description: {}", description);
    msg!("Stake: {} SOL", proposer_stake as f64 / 1e9);
    
    Ok(())
}

pub fn vote_on_proposal(
    ctx: Context<VoteOnProposal>,
    vote: bool, // true for approve, false for reject
    stake_amount: u64,
) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    let voter = &ctx.accounts.voter;
    
    proposal.add_vote(voter.key(), vote, stake_amount)?;
    
    let vote_type = if vote { "APPROVE" } else { "REJECT" };
    msg!("Vote recorded: {} with {} SOL stake", vote_type, stake_amount as f64 / 1e9);
    
    Ok(())
}

pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
    let proposal = &ctx.accounts.proposal;
    let whitelist = &mut ctx.accounts.whitelist;
    
    // Check if proposal passed
    require!(
        proposal.is_approved(),
        AmmError::ProposalNotApproved
    );
    
    // Add hook to whitelist
    whitelist.add_hook(proposal.hook_program_id)?;
    
    msg!("Proposal executed: Hook {} added to whitelist", proposal.hook_program_id);
    
    Ok(())
}

pub fn cancel_proposal(ctx: Context<CancelProposal>) -> Result<()> {
    let proposal = &mut ctx.accounts.proposal;
    
    require!(
        proposal.can_be_cancelled(),
        AmmError::ProposalCannotBeCancelled
    );
    
    proposal.cancel()?;
    
    msg!("Proposal cancelled: {}", proposal.hook_program_id);
    
    Ok(())
} 