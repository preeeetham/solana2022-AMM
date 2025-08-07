pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("5VFsZC9h31MA9gMkV8ycx8eeyHXJT4QE36SgopWKXnE7");

#[program]
pub mod token2022_amm {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }

    pub fn initialize_whitelist(ctx: Context<InitializeWhitelist>) -> Result<()> {
        instructions::whitelist::initialize_whitelist(ctx)
    }

    pub fn add_hook_to_whitelist(
        ctx: Context<AddHookToWhitelist>,
        hook_program_id: Pubkey,
    ) -> Result<()> {
        instructions::whitelist::add_hook_to_whitelist(ctx, hook_program_id)
    }

    pub fn remove_hook_from_whitelist(
        ctx: Context<RemoveHookFromWhitelist>,
        hook_program_id: Pubkey,
    ) -> Result<()> {
        instructions::whitelist::remove_hook_from_whitelist(ctx, hook_program_id)
    }

    pub fn validate_transfer_hook(
        ctx: Context<ValidateTransferHook>,
        hook_program_id: Pubkey,
    ) -> Result<bool> {
        instructions::whitelist::validate_transfer_hook(ctx, hook_program_id)
    }

    // AMM Pool Instructions
    pub fn initialize_pool(ctx: Context<InitializePool>) -> Result<()> {
        instructions::amm_pool::initialize_pool(ctx)
    }

    pub fn update_pool_config(
        ctx: Context<UpdatePoolConfig>,
        fee_rate: u64,
        min_liquidity: u64,
    ) -> Result<()> {
        instructions::amm_pool::update_pool_config(ctx, fee_rate, min_liquidity)
    }

    // Trading Instructions
    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::trading::swap(ctx, amount_in, min_amount_out)
    }

    pub fn swap_exact_tokens_for_tokens(
        ctx: Context<SwapExactTokensForTokens>,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<()> {
        instructions::trading::swap_exact_tokens_for_tokens(ctx, amount_in, min_amount_out)
    }

    // Liquidity Instructions
    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
        min_lp_tokens: u64,
    ) -> Result<()> {
        instructions::liquidity::add_liquidity(ctx, amount_a, amount_b, min_lp_tokens)
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        lp_tokens_to_burn: u64,
        min_token_a: u64,
        min_token_b: u64,
    ) -> Result<()> {
        instructions::liquidity::remove_liquidity(ctx, lp_tokens_to_burn, min_token_a, min_token_b)
    }

    // Governance Instructions
    pub fn create_hook_proposal(
        ctx: Context<CreateHookProposal>,
        hook_program_id: Pubkey,
        description: String,
        audit_report_url: String,
        proposer_stake: u64,
    ) -> Result<()> {
        instructions::governance::create_hook_proposal(
            ctx,
            hook_program_id,
            description,
            audit_report_url,
            proposer_stake,
        )
    }

    pub fn vote_on_proposal(
        ctx: Context<VoteOnProposal>,
        vote: bool,
        stake_amount: u64,
    ) -> Result<()> {
        instructions::governance::vote_on_proposal(ctx, vote, stake_amount)
    }

    pub fn execute_proposal(ctx: Context<ExecuteProposal>) -> Result<()> {
        instructions::governance::execute_proposal(ctx)
    }

    pub fn cancel_proposal(ctx: Context<CancelProposal>) -> Result<()> {
        instructions::governance::cancel_proposal(ctx)
    }
}
