# Token-2022 AMM with Transfer Hook Support

A secure Solana AMM (Automated Market Maker) that supports Token-2022 tokens with Transfer Hooks, enabling safe trading of tokens with custom transfer restrictions.

## Overview

This project implements a complete Token-2022 AMM solution that addresses the security challenges of trading tokens with Transfer Hooks. Traditional AMMs cannot safely handle Token-2022 tokens with Transfer Hooks because they may bypass or ignore transfer restrictions. This implementation provides:

- **Hook Whitelist Registry**: On-chain registry of authorized Transfer Hook programs
- **Token-2022 Integration**: Full support for Token-2022 transfers with hook validation
- **AMM Functionality**: Complete AMM with trading, liquidity provision, and pool management
- **Security**: Atomic transactions that respect transfer hook logic
- **SDK**: TypeScript SDK for easy integration

## Architecture

### Core Components

1. **Transfer Hook Whitelist Registry** (`TransferHookWhitelist`)
   - Stores authorized Transfer Hook program addresses
   - Role-based access control with authority management
   - Supports up to 32 whitelisted hooks

2. **AMM Pool** (`AmmPool`)
   - Constant product AMM implementation
   - Support for Token-2022 transfers with hook validation
   - Liquidity provision and removal
   - Fee management and slippage protection

3. **Safe Transfer Hook** (`SafeTransferHook`)
   - Example transfer hook implementation
   - Demonstrates basic validation and logging
   - Extensible architecture for custom security measures

4. **Trading Instructions**
   - `swap`: Execute token swaps with hook validation
   - `swap_exact_tokens_for_tokens`: Exact input/output swaps
   - `add_liquidity`: Add liquidity to pools
   - `remove_liquidity`: Remove liquidity from pools

## Features

### ✅ Implemented

- **Hook Whitelist Management**
  - Initialize whitelist with authority
  - Add/remove hooks from whitelist
  - Validate transfer hooks before transactions

- **Token-2022 Integration**
  - All transfers use Token-2022 program
  - Automatic hook validation during transfers
  - Support for both regular SPL and Token-2022 tokens

- **AMM Functionality**
  - Pool initialization and configuration
  - Trading with slippage protection
  - Liquidity provision and removal
  - Constant product formula implementation

- **Security Features**
  - Atomic transactions
  - Hook validation before transfers
  - Authority-based access control
  - Comprehensive error handling

- **Testing Infrastructure**
  - Integration tests for all functionality
  - Deployment and testing scripts
  - Example usage patterns

### 🔄 In Progress

- **SDK Development**
  - TypeScript SDK for easy integration
  - Helper functions for common operations
  - Documentation and examples

- **Documentation**
  - API reference
  - Developer guides
  - Security considerations

## Installation

### Prerequisites

- Node.js 16+
- Rust 1.70+
- Solana CLI 1.16+
- Anchor CLI 0.29.0

### Setup

1. Clone the repository:
```bash
git clone <repository-url>
cd token2022-amm
```

2. Install dependencies:
```bash
yarn install
```

3. Build the programs:
```bash
anchor build
```

4. Run tests:
```bash
anchor test
```

## Usage

### Basic Setup

1. **Deploy Programs**
```bash
anchor deploy
```

2. **Initialize Whitelist**
```typescript
const whitelistAccount = Keypair.generate();
await program.methods
  .initializeWhitelist()
  .accounts({
    whitelist: whitelistAccount.publicKey,
    authority: payer.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([whitelistAccount])
  .rpc();
```

3. **Add Transfer Hook to Whitelist**
```typescript
await program.methods
  .addHookToWhitelist(hookProgram.programId)
  .accounts({
    whitelist: whitelistAccount.publicKey,
    authority: payer.publicKey,
  })
  .rpc();
```

### Trading

1. **Execute Swap**
```typescript
await program.methods
  .swap(new BN(amountIn), new BN(minAmountOut))
  .accounts({
    pool: poolAddress,
    user: user.publicKey,
    userTokenA: userTokenAAccount,
    userTokenB: userTokenBAccount,
    poolTokenAVault: poolTokenAVault,
    poolTokenBVault: poolTokenBVault,
    tokenAMint: tokenAMint,
    tokenBMint: tokenBMint,
    whitelist: whitelistAddress,
    tokenProgram: TOKEN_PROGRAM_ID,
    token2022Program: TOKEN_2022_PROGRAM_ID,
  })
  .signers([user])
  .rpc();
```

2. **Add Liquidity**
```typescript
await program.methods
  .addLiquidity(
    new BN(amountA),
    new BN(amountB),
    new BN(minLpTokens)
  )
  .accounts({
    pool: poolAddress,
    user: user.publicKey,
    userTokenA: userTokenAAccount,
    userTokenB: userTokenBAccount,
    userLpToken: userLpTokenAccount,
    poolTokenAVault: poolTokenAVault,
    poolTokenBVault: poolTokenBVault,
    lpMint: lpMint,
    tokenAMint: tokenAMint,
    tokenBMint: tokenBMint,
    whitelist: whitelistAddress,
    tokenProgram: TOKEN_PROGRAM_ID,
    token2022Program: TOKEN_2022_PROGRAM_ID,
  })
  .signers([user])
  .rpc();
```

### SDK Usage

```typescript
import { createToken2022AmmSDK } from './sdk';

const sdk = createToken2022AmmSDK(connection, program, hookProgram, provider);

// Add hook to whitelist
await sdk.addHookToWhitelist(whitelistAddress, hookProgramId, authority);

// Execute swap
const tx = await sdk.swap(
  poolAddress,
  amountIn,
  minAmountOut,
  userTokenAAccount,
  userTokenBAccount,
  poolTokenAVault,
  poolTokenBVault,
  tokenAMint,
  tokenBMint,
  whitelistAddress,
  user
);
```

## Security Considerations

### Transfer Hook Validation

- All Token-2022 transfers validate hooks against the whitelist
- Unauthorized hooks are rejected before transfer execution
- Atomic transactions ensure consistency

### Access Control

- Whitelist management requires authority signature
- Pool operations require proper authorization
- Comprehensive error handling prevents unauthorized access

### Testing

- Full test coverage for all instructions
- Edge case testing for hook validation
- Integration tests with real Token-2022 tokens

## Development

### Project Structure

```
token2022-amm/
├── programs/
│   ├── token2022-amm/          # Main AMM program
│   └── safe-transfer-hook/     # Example transfer hook
├── sdk/                        # TypeScript SDK
├── tests/                      # Integration tests
├── scripts/                    # Deployment scripts
└── docs/                       # Documentation
```

### Key Files

- `programs/token2022-amm/src/state/whitelist.rs` - Whitelist implementation
- `programs/token2022-amm/src/state/amm_pool.rs` - AMM pool logic
- `programs/token2022-amm/src/instructions/trading.rs` - Trading instructions
- `programs/token2022-amm/src/instructions/liquidity.rs` - Liquidity management
- `sdk/index.ts` - TypeScript SDK

### Building

```bash
# Build programs
anchor build

# Run tests
anchor test

# Deploy to devnet
anchor deploy --provider.cluster devnet
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

This project is licensed under the MIT License.

## Support

For questions and support:
- Create an issue on GitHub
- Check the documentation in `/docs`
- Review the test files for usage examples 