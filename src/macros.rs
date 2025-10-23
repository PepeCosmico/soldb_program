#[macro_export]
macro_rules! require {
    ($cond:expr, $err:expr) => {
        if !$cond {
            ::solana_program::msg!(
                "{}",
                ::solana_program_error::ProgramError::from($err)
                    .to_str::<$crate::error::SolDbError>()
            );
            return Err($err.into());
        }
    };
}

#[macro_export]
macro_rules! require_keys_eq {
    ($a:expr, $b:expr, $err:expr) => {
        if $a != $b {
            return Err($err.into());
        }
    };
}
