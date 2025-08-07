use anchor_lang::prelude::*;
use crate::error::AmmError;

/// AMM Pool State
/// Manages liquidity pools for Token-2022 trading pairs
#[account]
#[derive(Default)]
pub struct AmmPool {
    /// Pool authority
    pub authority: Pubkey,
    
    /// Token A mint (e.g., SOL)
    pub token_a_mint: Pubkey,
    
    /// Token B mint (e.g., Token-2022)
    pub token_b_mint: Pubkey,
    
    /// Pool's token A vault
    pub token_a_vault: Pubkey,
    
    /// Pool's token B vault
    pub token_b_vault: Pubkey,
    
    /// LP token mint
    pub lp_mint: Pubkey,
    
    /// Total LP tokens minted
    pub total_lp_supply: u64,
    
    /// Token A reserve amount
    pub token_a_reserve: u64,
    
    /// Token B reserve amount
    pub token_b_reserve: u64,
    
    /// Fee rate (basis points, e.g., 30 = 0.3%)
    pub fee_rate: u64,
    
    /// Minimum liquidity required
    pub min_liquidity: u64,
    
    /// Pool bump seed
    pub bump: u8,
    
    /// Reserved for future use
    pub reserved: [u64; 8],
}

impl AmmPool {
    /// Initialize a new AMM pool
    pub fn initialize(
        &mut self,
        authority: Pubkey,
        token_a_mint: Pubkey,
        token_b_mint: Pubkey,
        token_a_vault: Pubkey,
        token_b_vault: Pubkey,
        lp_mint: Pubkey,
    ) -> Result<()> {
        self.authority = authority;
        self.token_a_mint = token_a_mint;
        self.token_b_mint = token_b_mint;
        self.token_a_vault = token_a_vault;
        self.token_b_vault = token_b_vault;
        self.lp_mint = lp_mint;
        self.total_lp_supply = 0;
        self.token_a_reserve = 0;
        self.token_b_reserve = 0;
        self.fee_rate = 30; // 0.3% default fee
        self.min_liquidity = 1000; // Minimum liquidity
        self.bump = 0; // Will be set by PDA
        self.reserved = [0u64; 8];
        Ok(())
    }
    
    /// Update pool configuration
    pub fn update_config(&mut self, fee_rate: u64, min_liquidity: u64) -> Result<()> {
        self.fee_rate = fee_rate;
        self.min_liquidity = min_liquidity;
        Ok(())
    }
    
    /// Calculate swap output using constant product formula
    pub fn calculate_swap_output(&self, amount_in: u64) -> Result<u64> {
        require!(amount_in > 0, AmmError::InvalidAmount);
        require!(self.token_a_reserve > 0, AmmError::InsufficientLiquidity);
        require!(self.token_b_reserve > 0, AmmError::InsufficientLiquidity);
        
        // Calculate fee
        let fee_amount = (amount_in * self.fee_rate) / 10000;
        let amount_in_after_fee = amount_in - fee_amount;
        
        // Constant product formula: (x + dx) * (y - dy) = x * y
        // dy = (y * dx) / (x + dx)
        let amount_out = (self.token_b_reserve * amount_in_after_fee) / 
                        (self.token_a_reserve + amount_in_after_fee);
        
        require!(amount_out > 0, AmmError::InsufficientOutputAmount);
        require!(amount_out < self.token_b_reserve, AmmError::InsufficientLiquidity);
        
        Ok(amount_out)
    }
    
    /// Calculate LP tokens for liquidity addition
    pub fn calculate_lp_tokens_for_liquidity(&self, amount_a: u64, amount_b: u64) -> Result<u64> {
        require!(amount_a > 0, AmmError::InvalidAmount);
        require!(amount_b > 0, AmmError::InvalidAmount);
        
        if self.total_lp_supply == 0 {
            // First liquidity provider
            let lp_tokens = (amount_a * amount_b).isqrt();
            require!(lp_tokens >= self.min_liquidity, AmmError::InsufficientLPTokens);
            Ok(lp_tokens)
        } else {
            // Calculate based on proportion of reserves
            let lp_tokens_a = (amount_a * self.total_lp_supply) / self.token_a_reserve;
            let lp_tokens_b = (amount_b * self.total_lp_supply) / self.token_b_reserve;
            Ok(lp_tokens_a.min(lp_tokens_b))
        }
    }
    
    /// Calculate tokens for LP burn
    pub fn calculate_tokens_for_lp_burn(&self, lp_tokens_to_burn: u64) -> Result<(u64, u64)> {
        require!(lp_tokens_to_burn > 0, AmmError::InvalidAmount);
        require!(lp_tokens_to_burn <= self.total_lp_supply, AmmError::InsufficientLPTokens);
        
        let proportion = lp_tokens_to_burn as f64 / self.total_lp_supply as f64;
        let token_a_amount = (self.token_a_reserve as f64 * proportion) as u64;
        let token_b_amount = (self.token_b_reserve as f64 * proportion) as u64;
        
        Ok((token_a_amount, token_b_amount))
    }
    
    /// Update pool state after swap
    pub fn update_swap_state(&mut self, amount_in: u64, amount_out: u64) -> Result<()> {
        self.token_a_reserve += amount_in;
        require!(self.token_b_reserve >= amount_out, AmmError::InsufficientLiquidity);
        self.token_b_reserve -= amount_out;
        Ok(())
    }
    
    /// Add liquidity to pool
    pub fn add_liquidity(&mut self, amount_a: u64, amount_b: u64, lp_tokens: u64) -> Result<()> {
        self.token_a_reserve += amount_a;
        self.token_b_reserve += amount_b;
        self.total_lp_supply += lp_tokens;
        Ok(())
    }
    
    /// Remove liquidity from pool
    pub fn remove_liquidity(&mut self, amount_a: u64, amount_b: u64, lp_tokens: u64) -> Result<()> {
        require!(self.token_a_reserve >= amount_a, AmmError::InsufficientLiquidity);
        require!(self.token_b_reserve >= amount_b, AmmError::InsufficientLiquidity);
        require!(self.total_lp_supply >= lp_tokens, AmmError::InsufficientLPTokens);
        
        self.token_a_reserve -= amount_a;
        self.token_b_reserve -= amount_b;
        self.total_lp_supply -= lp_tokens;
        Ok(())
    }
    
    /// Get current price ratio
    pub fn get_price_ratio(&self) -> Result<f64> {
        require!(self.token_a_reserve > 0, AmmError::InsufficientLiquidity);
        require!(self.token_b_reserve > 0, AmmError::InsufficientLiquidity);
        
        Ok(self.token_b_reserve as f64 / self.token_a_reserve as f64)
    }
    
    /// Get pool information
    pub fn get_pool_info(&self) -> (u64, u64, u64) {
        (self.token_a_reserve, self.token_b_reserve, self.total_lp_supply)
    }
} 