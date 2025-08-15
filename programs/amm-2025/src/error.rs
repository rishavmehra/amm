use anchor_lang::prelude::*;
use constant_product_curve::CurveError;

#[error_code]
pub enum AMMErrorCode {
    #[msg("Fee is too high")]
    InvalidFee,
    #[msg("zero balance")]
    ZeroBalance,
    #[msg("Pool is locked")]
    PoolLocked,
    #[msg("this offer is expired")]
    OfferExpired,
    #[msg("balance is zero")]
    BalanceZero,
    #[msg("invlaid authority")]
    InvalidAuth,
    #[msg("not authority is set")]
    NoAuthSet,
    #[msg("invalid precision")]
    InvalidPrecision,
    #[msg("overflow")]
    Overflow,
    #[msg("underflow")]
    Underflow,
    #[msg("insufficient balance")]
    InsufficientBalance,
    #[msg("slippage limit exceeded")]
    SlippageExceeded,
}


impl From<CurveError> for AMMErrorCode {
    fn from(error: CurveError) -> AMMErrorCode {
        match error {
            CurveError::InvalidPrecision => AMMErrorCode::InvalidPrecision,
            CurveError::Overflow => AMMErrorCode::Overflow,
            CurveError::Underflow => AMMErrorCode::Underflow,
            CurveError::InvalidFeeAmount => AMMErrorCode::InvalidFee,
            CurveError::InsufficientBalance => AMMErrorCode::InsufficientBalance,
            CurveError::ZeroBalance => AMMErrorCode::ZeroBalance,
            CurveError::SlippageLimitExceeded => AMMErrorCode::SlippageExceeded,
        }
    }
}