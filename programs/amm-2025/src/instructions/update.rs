use anchor_lang::prelude::*;

use crate::{state::Config};


#[derive(Accounts)]
pub struct Update<'info>{
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds=[
            b"config",
            config.seed.to_le_bytes().as_ref()
        ],
        bump = config.config_bump
    )]
    pub config: Account<'info, Config>,
    pub system_program: Program<'info, System>
}

impl<'info> Update<'info>  {
    pub fn lock(&mut self) -> Result<()> {
        require_keys_eq!(
            self.config.authority,
            self.user.key(),
            crate::error::AMMErrorCode::InvalidAuth
        );

        self.config.locked = true;
        Ok(())
    }

    pub fn unlock(&mut self) -> Result<()> {
        require_keys_eq!(
            self.config.authority,
            self.user.key(),
            crate::error::AMMErrorCode::InvalidAuth
        );

        self.config.locked = false;
        Ok(())
    }
}



