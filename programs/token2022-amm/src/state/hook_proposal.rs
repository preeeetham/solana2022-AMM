use anchor_lang::prelude::*;

#[account]
pub struct HookProposal {
    pub proposer: Pubkey,
    pub hook_program_id: Pubkey,
    pub description: String,
    pub audit_report_url: String,
    pub proposer_stake: u64,
    pub created_at: i64,
    pub voting_deadline: i64,
    pub status: ProposalStatus,
    pub total_approve_stake: u64,
    pub total_reject_stake: u64,
    pub votes: Vec<Vote>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    Active,
    Approved,
    Rejected,
    Cancelled,
    Executed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Vote {
    pub voter: Pubkey,
    pub vote: bool, // true for approve, false for reject
    pub stake_amount: u64,
    pub timestamp: i64,
}

impl HookProposal {
    pub const VOTING_PERIOD: i64 = 7 * 24 * 60 * 60; // 7 days in seconds
    pub const MIN_APPROVE_STAKE: u64 = 100 * 1_000_000_000; // 100 SOL minimum
    pub const MIN_PROPOSER_STAKE: u64 = 10 * 1_000_000_000; // 10 SOL minimum

    pub fn initialize(
        &mut self,
        proposer: Pubkey,
        hook_program_id: Pubkey,
        description: String,
        audit_report_url: String,
        proposer_stake: u64,
        created_at: i64,
    ) -> Result<()> {
        require!(
            proposer_stake >= Self::MIN_PROPOSER_STAKE,
            AmmError::InsufficientProposerStake
        );

        self.proposer = proposer;
        self.hook_program_id = hook_program_id;
        self.description = description;
        self.audit_report_url = audit_report_url;
        self.proposer_stake = proposer_stake;
        self.created_at = created_at;
        self.voting_deadline = created_at + Self::VOTING_PERIOD;
        self.status = ProposalStatus::Active;
        self.total_approve_stake = 0;
        self.total_reject_stake = 0;
        self.votes = Vec::new();

        Ok(())
    }

    pub fn add_vote(&mut self, voter: Pubkey, vote: bool, stake_amount: u64) -> Result<()> {
        require!(
            self.status == ProposalStatus::Active,
            AmmError::ProposalNotActive
        );

        require!(
            Clock::get()?.unix_timestamp < self.voting_deadline,
            AmmError::VotingPeriodExpired
        );

        // Check if voter already voted
        for existing_vote in &self.votes {
            require!(
                existing_vote.voter != voter,
                AmmError::VoterAlreadyVoted
            );
        }

        let vote_record = Vote {
            voter,
            vote,
            stake_amount,
            timestamp: Clock::get()?.unix_timestamp,
        };

        self.votes.push(vote_record);

        if vote {
            self.total_approve_stake = self.total_approve_stake.checked_add(stake_amount)
                .ok_or(AmmError::StakeOverflow)?;
        } else {
            self.total_reject_stake = self.total_reject_stake.checked_add(stake_amount)
                .ok_or(AmmError::StakeOverflow)?;
        }

        Ok(())
    }

    pub fn is_executable(&self) -> bool {
        self.status == ProposalStatus::Approved
    }

    pub fn is_approved(&self) -> bool {
        self.total_approve_stake >= Self::MIN_APPROVE_STAKE
    }

    pub fn can_be_cancelled(&self) -> bool {
        self.status == ProposalStatus::Active && 
        Clock::get().unwrap().unix_timestamp < self.voting_deadline
    }

    pub fn cancel(&mut self) -> Result<()> {
        require!(
            self.status == ProposalStatus::Active,
            AmmError::ProposalNotActive
        );

        self.status = ProposalStatus::Cancelled;
        Ok(())
    }

    pub fn finalize(&mut self) -> Result<()> {
        require!(
            self.status == ProposalStatus::Active,
            AmmError::ProposalNotActive
        );

        require!(
            Clock::get()?.unix_timestamp >= self.voting_deadline,
            AmmError::VotingPeriodNotExpired
        );

        if self.is_approved() {
            self.status = ProposalStatus::Approved;
        } else {
            self.status = ProposalStatus::Rejected;
        }

        Ok(())
    }

    pub fn get_vote_summary(&self) -> (u64, u64, u64) {
        (
            self.total_approve_stake,
            self.total_reject_stake,
            self.votes.len() as u64,
        )
    }
}

// Error types for governance
#[error_code]
pub enum AmmError {
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