
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Token, Mint, MintTo, TokenAccount, mint_to},
    token_interface::{transfer_checked, TransferChecked}
};
use constant_product_curve::ConstantProduct;

use crate::{error::AMMErrorCode, state::config::Config};
use crate::{assert_not_locked, assert_not_expired, assert_non_zero};

#[derive(Accounts)]
pub struct  Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub mint_x: Box<Account<'info, Mint>>,
    pub mint_y: Box<Account<'info, Mint>>,
    #[account(
        mut, 
        seeds=[b"liquidity", config.key().as_ref()],
        bump=config.lp_bump
    )]
    pub mint_lp: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint=config.mint_x,
        associated_token::authority=auth,
    )]
    pub vault_x:Box<Account<'info, TokenAccount>>,

    #[account(
        mut, 
        associated_token::mint= config.mint_y,
        associated_token::authority=auth
    )]
    pub vault_y: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = config.mint_x,
        associated_token::authority = user 
    )]
    pub user_x: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint= config.mint_y,
        associated_token::authority= user
    )]
    pub user_y: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer= user,
        associated_token::mint = mint_lp,
        associated_token::authority = user
    )]
    pub user_lp: Box<Account<'info, TokenAccount>>,

    /// CHECK: this is safe
    #[account(seeds=[b"auth"], bump=config.auth_bump)]
    pub auth: UncheckedAccount<'info>,

    #[account(
        has_one=mint_x,
        has_one=mint_y,
        seeds = [
            b"config",
            config.seed.to_le_bytes().as_ref()
        ],
        bump=config.config_bump,
    )]
    pub config: Account<'info, Config>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(
        &self,
        amount: u64,
        max_x: u64,
        max_y: u64,
        expiration: i64,
    ) -> Result<()> {
        assert_not_locked!(self.config.locked);
        assert_not_expired!(expiration);
        assert_non_zero!([amount, max_x, max_y]);

        let (x,y) = match self.mint_lp.supply == 0 && self.vault_x.amount == 0 && self.vault_y.amount == 0 {
            true => (max_x, max_y),
            false => {
                let amounts = ConstantProduct::xy_deposit_amounts_from_l(
                    self.vault_x.amount,
                    self.vault_y.amount,
                    self.mint_lp.supply,
                    amount,
                    6
                ).map_err(AMMErrorCode::from)?;
                (amounts.x, amounts.y)
            }
        };
        
        self.deposit_tokens(true, max_x)?;
        self.deposit_tokens(false, max_y)?;
        self.mint_lp_token(amount)
    }


    pub fn deposit_tokens(&self, is_x: bool, amount: u64) -> Result<()> {
        let (from, to, mint, decimals) = match is_x{
            true => (self.user_x.to_account_info(), self.vault_x.to_account_info(), self.mint_x.to_account_info(), self.mint_x.decimals),
            false => (self.user_y.to_account_info(), self.vault_y.to_account_info(), self.mint_y.to_account_info(), self.mint_y.decimals),
        };

        let cpi_accounts = TransferChecked {
            from,
            mint,
            to,
            authority: self.user.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        transfer_checked(ctx,amount, decimals)
    }

    pub fn mint_lp_token(
        &self,
        amount:u64
    ) -> Result<()> {
        let accounts = MintTo {
            mint: self.mint_lp.to_account_info(),
            to: self.user_lp.to_account_info(),
            authority: self.config.to_account_info(),
        };

        let seeds = &[
            &b"auth"[..],
            &[self.config.auth_bump]
        ];

        let signer_seeds= &[&seeds[..]];
        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), accounts, signer_seeds);
        
        mint_to(ctx, amount)
    }
}

