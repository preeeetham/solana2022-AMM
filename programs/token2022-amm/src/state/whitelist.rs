use anchor_lang::prelude::*;
use crate::error::AmmError;

/// Maximum number of whitelisted transfer hook programs
pub const MAX_WHITELISTED_HOOKS: usize = 32;

/// Transfer Hook Whitelist Configuration
/// This structure stores a list of trusted Transfer Hook program IDs
/// that are allowed to be used with Token-2022 assets in this AMM
#[account]
#[derive(Default)]
pub struct TransferHookWhitelist {
    /// Authority that can modify this whitelist
    pub authority: Pubkey,
    /// Number of currently whitelisted hook programs
    pub hook_count: u32,
    /// Reserved for future use
    pub reserved: u32,
    /// Array of whitelisted Transfer Hook program IDs
    pub whitelisted_hooks: [Pubkey; MAX_WHITELISTED_HOOKS],
    /// Padding for future expansion
    pub padding: [u64; 8],
}

impl TransferHookWhitelist {
    /// Initialize a new whitelist with the given authority
    pub fn initialize(&mut self, authority: Pubkey) -> Result<()> {
        self.authority = authority;
        self.hook_count = 0;
        self.reserved = 0;
        self.whitelisted_hooks = [Pubkey::default(); MAX_WHITELISTED_HOOKS];
        self.padding = [0u64; 8];
        Ok(())
    }

    /// Check if a Transfer Hook program ID is whitelisted
    pub fn is_hook_whitelisted(&self, hook_program_id: &Pubkey) -> bool {
        for i in 0..(self.hook_count as usize) {
            if self.whitelisted_hooks[i] == *hook_program_id {
                return true;
            }
        }
        false
    }

    /// Add a Transfer Hook program ID to the whitelist
    pub fn add_hook(&mut self, hook_program_id: Pubkey) -> Result<()> {
        if self.hook_count >= MAX_WHITELISTED_HOOKS as u32 {
            return Err(AmmError::WhitelistFull.into());
        }

        // Check if already whitelisted
        if self.is_hook_whitelisted(&hook_program_id) {
            return Err(AmmError::HookAlreadyWhitelisted.into());
        }

        self.whitelisted_hooks[self.hook_count as usize] = hook_program_id;
        self.hook_count += 1;
        Ok(())
    }

    /// Remove a Transfer Hook program ID from the whitelist
    pub fn remove_hook(&mut self, hook_program_id: &Pubkey) -> Result<()> {
        for i in 0..(self.hook_count as usize) {
            if self.whitelisted_hooks[i] == *hook_program_id {
                // Shift remaining elements left
                for j in i..(self.hook_count as usize - 1) {
                    self.whitelisted_hooks[j] = self.whitelisted_hooks[j + 1];
                }
                // Clear the last element
                self.whitelisted_hooks[self.hook_count as usize - 1] = Pubkey::default();
                self.hook_count -= 1;
                return Ok(());
            }
        }
        Err(AmmError::HookNotWhitelisted.into())
    }
}