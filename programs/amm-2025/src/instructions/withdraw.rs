use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{burn, transfer_checked, Burn, Mint, Token, TokenAccount, Transfer, TransferChecked},
};
use constant_product_curve::ConstantProduct;

use crate::{assert_non_zero, assert_not_expired, assert_not_locked, error::AMMErrorCode, state::Config};

#[derive(Accounts)]
pub struct Withdraw<'info> {
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
        associated_token::mint=mint_x,
        associated_token::authority=auth
    )]
    pub vault_x: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint=mint_y,
        associated_token::authority=auth
    )]
    pub vault_y: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_x,
        associated_token::authority=user,
    )]
    pub user_x: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint_y,
        associated_token::authority=user,
    )]
    pub user_y: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer=user,
        associated_token::mint=mint_lp,
        associated_token::authority=user
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
        bump= config.config_bump
    )]
    pub config: Account<'info, Config>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,

}


impl<'info> Withdraw<'info> {
    pub fn withdraw(
        &self,
        amount: u64,
        min_x: u64,
        min_y: u64,
        expiration: i64
    ) -> Result<()> {
        assert_not_locked!(self.config.locked);
        assert_not_expired!(expiration);
        assert_non_zero!([amount]);

        let amounts = ConstantProduct::xy_withdraw_amounts_from_l(
            self.vault_x.amount,
            self.vault_y.amount,
            self.mint_lp.supply,
            amount,
            6
        ).map_err(AMMErrorCode::from)?;

        // Check for slippage
        require!(min_x <= amounts.x && min_y <= amounts.y, AMMErrorCode::SlippageExceeded);
        
        self.withdraw_tokens(true, amounts.x)?;
        self.withdraw_tokens(false, amounts.y)?;
        self.burn_lp_tokens(amount)
    }

    pub fn withdraw_tokens(
        &self,
        x: bool,
        amount: u64
    ) -> Result<()>{
        let (from, to, mint, decimals) = match x {
            true => (self.vault_x.to_account_info(), self.user_x.to_account_info(), self.mint_x.to_account_info(), self.mint_x.decimals),
            false => (self.vault_y.to_account_info(), self.user_y.to_account_info(), self.mint_y.to_account_info(), self.mint_y.decimals),
        };

        let cpi_accounts = TransferChecked {
            from,
            mint,
            to,
            authority: self.user.to_account_info(),
        };

        let ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        transfer_checked(ctx, amount, decimals)
    }

    pub fn burn_lp_tokens(
        &self,
        amount: u64
    ) -> Result<()> {
        let cpi_accounts = Burn {
            mint: self.mint_lp.to_account_info(),
            from: self.user_lp.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
        
        burn(cpi_ctx, amount)
    }
}