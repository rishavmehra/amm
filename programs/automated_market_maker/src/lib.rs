use anchor_lang::prelude::*;

declare_id!("GuFNA9R65c3LCEkefgfhTEK3Epnz2iU8kULUZrnhXumK");

#[program]
pub mod automated_market_maker {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
