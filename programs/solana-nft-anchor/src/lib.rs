use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_master_edition_v3, create_metadata_accounts_v3, CreateMasterEditionV3,
        CreateMetadataAccountsV3, Metadata, 
    }, 
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};
use mpl_token_metadata::{
    pda::{find_master_edition_account, find_metadata_account},
    state::DataV2,
};

declare_id!("BZC28tbriJNMVB1WpAsiAywUUQUCm7q6JfbzeTfXXgtz"); // shouldn't be similar to mine

#[program]
pub mod solana_nft_anchor {

    use super::*;

    // init_nft - creates a new NFT
    pub fn init_nft(
        ctx: Context<InitNFT>,
        name: String,  
        symbol: String, 
        uri: String,
    ) -> Result<()> {
        // create mint account
        let cpi_context = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.associated_token_account.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        );

        mint_to(cpi_context, 1)?;

        // create metadata account
        let cpi_context = CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMetadataAccountsV3 {
                metadata: ctx.accounts.metadata_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                mint_authority: ctx.accounts.signer.to_account_info(),
                update_authority: ctx.accounts.signer.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        );

        let data_v2 = DataV2 {
            name: name,
            symbol: symbol,
            uri: uri,
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        create_metadata_accounts_v3(cpi_context, data_v2, false, true, None)?;

        //create master edition account
        let cpi_context = CpiContext::new(
            ctx.accounts.token_metadata_program.to_account_info(),
            CreateMasterEditionV3 {
                edition: ctx.accounts.master_edition_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.signer.to_account_info(),
                mint_authority: ctx.accounts.signer.to_account_info(),
                payer: ctx.accounts.signer.to_account_info(),
                metadata: ctx.accounts.metadata_account.to_account_info(),
                token_program: ctx.accounts.token_program.to_account_info(),
                system_program: ctx.accounts.system_program.to_account_info(),
                rent: ctx.accounts.rent.to_account_info(),
            },
        );

        create_master_edition_v3(cpi_context, None)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitNFT<'info> {
    // We can mark the account as a signer using the `signer` attribute
    // We can mark the account as mutable using the `mut` attribute
    //   - This allows us to modify the account (paying fees)

    // Anchor constraints
    // #[account(<constraints>)]
    //   - These are built-in features that Anchor provides to simplify common
    //     security checks; mutable accounts, signer accounts, etc.

    // The check below is crucial when using the AccountInfo wrapper to ensure
    //   we are passing in the correct account
    /// CHECK: ok, we are passing in this account ourselves
    #[account(mut, signer)]
    pub signer: AccountInfo<'info>,
    
    
    // MINT ACCOUNT
    // contains details about the token such as mint authority, freeze authority,
    //   total supply …etc.
    // init - wrapper around `system_instruction::create_account()`
    //   - allocate space for the account
    //   - transfer lamports for rent (fees)
    //   - assigning account; link to the appropriate owning program
    // payer = signer
    //   - used to pay the rent to store data
    //   - incentivizes validators
    //   - without rent, data is pruned from blockchain
    // mint::decimals = 0
    //   - decimals for NFT
    //   - cannot have a fraction of an NFT
    // mint::authority = signer.key()
    //   - authority to mint new tokens
    //   - signer is the authority
    // mint::freeze_authority = signer.key()
    //   - authority to freeze tokens
    //   - signer is the authority
    #[account(
        init,
        payer = signer,
        mint::decimals = 0,
        mint::authority = signer.key(),
        mint::freeze_authority = signer.key(),
    )]
    pub mint: Account<'info, Mint>,
    
    
    // ASSOCIATED TOKEN ACCOUNT
    // a special slot for storing a specific type of token
    // init_if_needed - creates an account in the wallet if no token account exists
    // payer - account that pays for the account creation; signer
    // associated_token::mint = mint - the mint account that the token is associated with
    // associated_token::authority = signer - the authority to transfer tokens
    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = signer
    )]
    pub associated_token_account: Account<'info, TokenAccount>,
    
    
    
    
    /// CHECK - address
    #[account(
        mut,
        address=find_metadata_account(&mint.key()).0,
    )]
    pub metadata_account: AccountInfo<'info>, 
    
    
    
    
    /// CHECK: address
    #[account(
        mut,
        address=find_master_edition_account(&mint.key()).0,
    )]
    pub master_edition_account: AccountInfo<'info>,


    // PROGRAMS

    // Token program
    pub token_program: Program<'info, Token>,
    // handles creation of our ATA (associated token account)
    pub associated_token_program: Program<'info, AssociatedToken>,
    // handles creation of our metadata account
    pub token_metadata_program: Program<'info, Metadata>,
    // associated token program may need to create a new ATA
    // responsible for creating all accounts
    pub system_program: Program<'info, System>,
    // Solana requires rent-exempt (2 years of SOL deposit)
    pub rent: Sysvar<'info, Rent>,
}
