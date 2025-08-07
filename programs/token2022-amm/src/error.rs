use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Custom error message")]
    CustomError,
    
    #[msg("Invalid whitelist authority")]
    InvalidWhitelistAuthority,
    
    #[msg("Invalid pool authority")]
    InvalidPoolAuthority,
    
    #[msg("Hook not whitelisted")]
    HookNotWhitelisted,
    
    #[msg("Hook already whitelisted")]
    HookAlreadyWhitelisted,
    
    #[msg("Whitelist is full")]
    WhitelistFull,
    
    #[msg("Insufficient output amount")]
    InsufficientOutputAmount,
    
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    
    #[msg("Insufficient LP tokens")]
    InsufficientLPTokens,
    
    #[msg("Insufficient token A")]
    InsufficientTokenA,
    
    #[msg("Insufficient token B")]
    InsufficientTokenB,
    
    #[msg("Invalid amount")]
    InvalidAmount,
    
    #[msg("Invalid slippage tolerance")]
    InvalidSlippageTolerance,
    
    #[msg("Pool already exists")]
    PoolAlreadyExists,
    
    #[msg("Pool not found")]
    PoolNotFound,
    
    #[msg("Invalid token pair")]
    InvalidTokenPair,
    
    #[msg("Insufficient proposer stake")]
    InsufficientProposerStake,
    
    #[msg("Proposal not active")]
    ProposalNotActive,
    
    #[msg("Voting period expired")]
    VotingPeriodExpired,
    
    #[msg("Voter already voted")]
    VoterAlreadyVoted,
    
    #[msg("Stake overflow")]
    StakeOverflow,
    
    #[msg("Proposal not executable")]
    ProposalNotExecutable,
    
    #[msg("Proposal not approved")]
    ProposalNotApproved,
    
    #[msg("Proposal cannot be cancelled")]
    ProposalCannotBeCancelled,
    
    #[msg("Voting period not expired")]
    VotingPeriodNotExpired,
    
    #[msg("Invalid proposal proposer")]
    InvalidProposalProposer,
}
