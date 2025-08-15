use std::result;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked}};
use constant_product_curve::{ConstantProduct, LiquidityPair};

use crate::{assert_non_zero, assert_not_expired, assert_not_locked, error::AMMErrorCode, state::Config};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub mint_x: Box<Account<'info, Mint>>,
    pub mint_y: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer=user,
        associated_token::mint=mint_x,
        associated_token::authority=auth,
    )]
    pub user_x: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer=user,
        associated_token::mint=mint_y,
        associated_token::authority=auth,
    )]
    pub user_y: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint=mint_x,
        associated_token::authority=auth,
    )]
    pub vault_x: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint=mint_y,
        associated_token::authority=auth,
    )]
    pub vault_y: Box<Account<'info, TokenAccount>>,

    /// CHECKED: this is the safe account only for the sign the transaction - actually more secure
    #[account(
        seeds=[b"auth"],
        bump=config.auth_bump
    )]
    pub auth: UncheckedAccount<'info>,

    #[account(
        has_one=mint_x,
        has_one=mint_y,
        seeds=[
            b"config",
            config.seed.to_le_bytes().as_ref()
        ],
        bump= config.config_bump
    )]
    pub config: Account<'info, Config>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

}

impl<'info> Swap<'info> {
    pub fn swap(
        &mut self,
        x: bool,
        amount: u64,
        min: u64,
        expiration: i64
    ) -> Result<()> {
        assert_not_locked!(self.config.locked);
        assert_not_expired!(expiration);
        assert_non_zero!([amount]);

        let mut curve = ConstantProduct::init(
            self.vault_x.amount,
            self.vault_y.amount,
            self.vault_x.amount,
            self.config.fee,
            None
        ).map_err(AMMErrorCode::from)?;

        let p = match x {
            true => LiquidityPair::X,
            false => LiquidityPair::Y
        };

        let res = curve.swap(p, amount, min).map_err(AMMErrorCode::from)?;

        assert_non_zero!([res.deposit, res.withdraw]);

        self.deposit_token(x, amount)?;
        self.withdraw_token(x, amount)?;
        Ok(())
    }

    pub fn deposit_token(
        &mut self, 
        x: bool,
        amount: u64
    ) -> Result<()>{
        let(from, to, mint, decimals) = match x {
            true => (self.user_x.to_account_info(), self.vault_x.to_account_info(), self.mint_x.to_account_info(), self.mint_x.decimals),
            false => (self.user_y.to_account_info(), self.user_y.to_account_info(), self.mint_y.to_account_info(), self.mint_y.decimals)
        };

        let cpi_accounts = TransferChecked{
            from: from,
            to: to,
            mint: mint,
            authority: self.user.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        transfer_checked(cpi_ctx, amount, decimals)
    }

    pub fn  withdraw_token(
        &mut self,
        x: bool,
        amount: u64
    ) -> Result<()> {
        let(from, to, mint, decimals) = match x {
            true => (self.user_x.to_account_info(), self.vault_x.to_account_info(), self.mint_x.to_account_info(), self.mint_x.decimals),
            false => (self.user_y.to_account_info(), self.vault_x.to_account_info(), self.mint_x.to_account_info(), self.mint_x.decimals)
        };

        let cpi_accounts = TransferChecked {
            from: from,
            to: to,
            mint: mint,
            authority: self.user.to_account_info()
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        transfer_checked(cpi_ctx, amount, decimals)
    }
}

