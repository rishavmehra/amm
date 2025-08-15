
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}};

use crate::state::config::Config;
use crate::error::AMMErrorCode;

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct Initialize<'info>{
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub mint_x: Account<'info, Mint>,
    pub mint_y: Account<'info, Mint>,
    #[account(
        init,
        seeds = [b"liquidity",config.key.as_ref()],
        payer=initializer,
        bump,
        mint::decimals=6,
        mint::authority=auth
    )]
    pub mint_lp: Account<'info, Mint>,

    #[account(
        init,
        payer = initializer,
        associated_token::mint=mint_x,
        associated_token::authority=auth,
    )]
    pub vault_x: Account<'info, TokenAccount>,
    #[account(
        init,
        payer = initializer,
        associated_token::mint=mint_y,
        associated_token::authority=auth,
    )]
    pub vault_y: Account<'info, TokenAccount>,

    /// CHECKED: this is safer
    #[account(seeds=[b"auth"], bump)]
    pub auth: UncheckedAccount<'info>,

    #[account(
        init,
        payer=initializer,
        seeds=[b"config", seed.to_le_bytes().as_ref()],
        bump,
        space= 8 + Config::INIT_SPACE
    )]
    pub config: Account<'info, Config>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>
}

impl<'info> Initialize<'info> {
    pub fn init (
        &mut self,
        bumps: InitializeBumps,
        seeds: u64,
        fee: u16,
        authority: Pubkey,
    ) -> Result<()> {

        require!(fee <= 10000, AMMErrorCode::InvalidFee);
        
        let(auth_bump, config_bump, lp_bump) = (
            &bumps.auth, &bumps.config, &bumps.mint_lp
        );

        self.config.init(
            seeds,
            authority,
            self.mint_x.key(),
            self.mint_y.key(),
            fee,
            *auth_bump,
            *config_bump,
            *lp_bump
        );
        Ok(())
    }
}
