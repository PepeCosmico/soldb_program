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

#[macro_export]
macro_rules! require_signer {
    ($a:expr) => {
        if !$a.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
    };
}
#[macro_export]
macro_rules! require_writer {
    ($a:expr) => {
        if !$a.is_writable {
            let err = SolDbError::NotWritable;
            return Err(err.into());
        }
    };
}

#[macro_export]
macro_rules! require_is_empty {
    ($a:expr) => {
        let is_fresh = $a.lamports() == 0
            && $a.owner == &solana_system_interface::program::id()
            && $a.data_len() == 0;

        if !is_fresh {
            msg!(
                "PDA already initialized (lamports={}, owner={}, data_len={})",
                $a.lamports(),
                $a.owner,
                $a.data_len()
            );
            return Err(ProgramError::AccountAlreadyInitialized);
        }
    };
}

#[macro_export]
macro_rules! require_system_program {
    ($a:expr) => {
        if $a.key != &solana_system_interface::program::id() {
            return Err(ProgramError::IncorrectProgramId);
        }
    };
}
