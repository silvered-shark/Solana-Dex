use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, mint_to};
use spl_token_swap::state::SwapVersion;
// use spl_token_swap::processor::Processor;
// use spl_token_swap::instruction::Initialize;
use spl_token_swap::instruction::initialize;
use spl_token_swap::curve::fees;
use spl_token_swap::curve::base;
use std::sync::Arc;
use spl_token_swap::curve::constant_product::ConstantProductCurve;
use anchor_lang::solana_program::program::invoke;



declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod solana_dex {


    use super::*;

    // create Token Swap State Account
    // Create Token Mint Accounts
    // Find swap pool Authority PDA
    // Create ATAs
    // Mint tokens into the ATAs
    // Create Pool Token Mint Account, Pool Token Account, Pool Token Fee Account
    // Create swap pool via CPI


    pub fn initialize_token_swap_account(ctx: Context<InitializeTokenSwapAccount>) -> Result<()> {
        msg!("Created Token Swap State Account---");
        Ok(())
    }

    pub fn initialize_mint_accounts(ctx: Context<InitializeMintAccounts>) -> Result<()> {
        msg!("Created Token X Mint Account---");
        msg!("Created Token Y Mint Account---");
        Ok(())
    }

    pub fn initialize_token_accounts(ctx: Context<InitializeTokenAccounts>) -> Result<()> {
        msg!("Created Token X Account---");
        msg!("Created Token Y Account---");
        Ok(())
    }

    pub fn mint_tokens(ctx: Context<MintTokens>, amount: u64) -> Result<()> {

        // Account required for the CPI
        let cpi_accounts = anchor_spl::token::MintTo {
            mint: ctx.accounts.mint_account.to_account_info(), 
            to: ctx.accounts.token_account.to_account_info(), 
            authority: ctx.accounts.mint_authority.to_account_info(), 
        };

        // Program in which CPI will be invoked
        let cpi_program = ctx.accounts.token_program.to_account_info();

        // Create the CpiContext (All non-argument inputs)
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        // Call anchor's helper function, passing in the CPI context and amount(input arguement)
        mint_to(cpi_ctx, amount)?;
        
        msg!("Minted {} tokens to ATA---", amount);
        Ok(())
    }

    pub fn initialize_pool_accounts(ctx: Context<InitializePoolAccounts>) -> Result<()> {
        msg!("Created Pool Token Mint Account---");
        msg!("Created Pool Token Account---");
        msg!("Created Pool Token Fee Account---");
        Ok(())
    }

    pub fn initialize_swap_pool(ctx: Context<InitializeSwapPool>, bump:u8) -> Result<()> {

        let swap_fees = fees::Fees { 
            trade_fee_numerator: 0, 
            trade_fee_denominator: 10000, 
            owner_trade_fee_numerator: 5, 
            owner_trade_fee_denominator: 10000, 
            owner_withdraw_fee_numerator: 0, 
            owner_withdraw_fee_denominator: 0, 
            host_fee_numerator: 20, 
            host_fee_denominator: 100 
        };
        let swap_curve = base::SwapCurve {
            curve_type: base::CurveType::ConstantProduct, 
            calculator:  Arc::new(ConstantProductCurve {})
        };

        let initialize_instruction = initialize(
            ctx.accounts.token_swap_program.key, 
            ctx.accounts.token_program.key, 
            ctx.accounts.swap_pubkey.key, 
            ctx.accounts.authority_pubkey.key, 
            &ctx.accounts.token_x_account.key(), 
            &ctx.accounts.token_y_account.key(), 
            &ctx.accounts.pool_token_mint.key(), 
            &ctx.accounts.pool_token_fee_account.key(), 
            &ctx.accounts.pool_token_account.key(), 
            swap_fees, 
            swap_curve
        )?;

        msg!("Calling the token swap program to initialize a new pool...");
        invoke(
            &initialize_instruction,
            &[
                // 0. `[writable, signer]` New Token-swap to create.
                ctx.accounts.token_swap_account.to_account_info(),
                // 1. `[]` swap authority derived from `create_program_address(&[Token-swap account])`
                ctx.accounts.swap_pubkey.clone(),
                // 2. `[]` token_a Account. Must be non zero, owned by swap authority.
                ctx.accounts.token_x_account.to_account_info(),
                // 3. `[]` token_b Account. Must be non zero, owned by swap authority.
                ctx.accounts.token_y_account.to_account_info(), 
                // 4. `[writable]` Pool Token Mint. Must be empty, owned by swap authority.
                ctx.accounts.pool_token_mint.to_account_info(), 
                // 5. `[]` Pool Token Account to deposit trading and withdraw fees.
                ctx.accounts.pool_token_fee_account.to_account_info(), 
                // 6. `[writable]` Pool Token Account to deposit the initial pool token supply
                ctx.accounts.pool_token_account.to_account_info(), 
                // 7. `[]` Token program id
                ctx.accounts.token_program.to_account_info()
            ],
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeTokenSwapAccount<'info> {
    #[account(mut)]
    signer: Signer<'info>, 
    #[account(
        init, 
        payer=signer, 
        space=SwapVersion::LATEST_LEN, 
        owner=token_swap_program.key()
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    token_swap_account: AccountInfo<'info>, 
    /// CHECK: This is not dangerous because we don't read or write from this account
    token_swap_program: AccountInfo<'info>, 
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeMintAccounts<'info> {
    #[account(mut)]
    signer: Signer<'info>, 
    /// CHECK: This is not dangerous because we don't read or write from this account
    mint_authority: AccountInfo<'info>, 
    #[account(
        init, 
        payer=signer, 
        mint::decimals = 9, 
        mint::authority = mint_authority
    )]
    x_mint: Account<'info, Mint>,
    #[account(
        init, 
        payer=signer, 
        mint::decimals = 9, 
        mint::authority = mint_authority
    )]
    y_mint: Account<'info, Mint>,
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeTokenAccounts<'info> {
    #[account(mut)]
    signer: Signer<'info>, // Signer
    /// CHECK: This is not dangerous because we don't read or write from this account
    token_authority: AccountInfo<'info>, // Authority of the token Accounts
    x_mint: Account<'info, Mint>,  // X-mint
    y_mint: Account<'info, Mint>,  // Y-mint
    #[account(
        init,
        payer=signer,
        token::mint = x_mint,
        token::authority = token_authority,
    )]
    token_x_account: Account<'info, TokenAccount>, // Token X ATA
    #[account(
        init,
        payer=signer,
        token::mint = y_mint,
        token::authority = token_authority,
    )]
    token_y_account: Account<'info, TokenAccount>, // Token Y ATA
    token_program: Program<'info, Token>, // Token Program
    rent: Sysvar<'info, Rent>,     // Rent
    system_program: Program<'info, System>, // System Program
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(mut)]
    mint_authority: Signer<'info>, 
    #[account(mut)]
    mint_account: Account<'info, Mint>,
    #[account(mut)]
    token_account: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>
}

#[derive(Accounts)]
pub struct InitializePoolAccounts<'info> {
    #[account(mut)]
    signer: Signer<'info>, 
    /// CHECK: This is not dangerous because we don't read or write from this account
    mint_authority: AccountInfo<'info>, 
    /// CHECK: This is not dangerous because we don't read or write from this account
    fee_owner: AccountInfo<'info>, 
    #[account(
        init, 
        payer=signer, 
        mint::decimals = 9, 
        mint::authority = mint_authority
    )]
    pool_token_mint: Account<'info, Mint>, 
    #[account(
        init,
        payer=signer, 
        token::mint = pool_token_mint, 
        token::authority = signer
    )]
    pool_token_account: Account<'info, TokenAccount>, // Initial LP tokens 
    #[account(
        init, 
        payer=signer, 
        token::mint=pool_token_mint, 
        token::authority=fee_owner
    )]
    pool_token_fee_account: Account<'info, TokenAccount>, // Swap Fees
    token_program: Program<'info, Token>,
    rent: Sysvar<'info, Rent>,
    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeSwapPool<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    token_swap_program: AccountInfo<'info>, 
    token_program: Program<'info, Token>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    swap_pubkey: AccountInfo<'info>, 
    /// CHECK: This is not dangerous because we don't read or write from this account
    authority_pubkey: AccountInfo<'info>, 
    token_x_account: Box<Account<'info, TokenAccount>>, // Token X ATA
    token_y_account: Box<Account<'info, TokenAccount>>, // Token Y ATA
    #[account(mut)]
    pool_token_mint: Box<Account<'info, Mint>>, 
    pool_token_fee_account: Box<Account<'info, TokenAccount>>, // Swap Fees
    #[account(mut)]
    pool_token_account: Box<Account<'info, TokenAccount>>, // Initial LP tokens 
    #[account(mut)]
    token_swap_account: Signer<'info>,
}