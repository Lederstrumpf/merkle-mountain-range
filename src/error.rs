pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    GetRootOnEmpty,
    InconsistentlyStored(u64),
    StoreError(crate::string::String),
    /// proof items is not enough to build a tree
    CorruptedProof,
    /// The leaves is an empty list, or beyond the mmr range
    GenProofForInvalidLeaves,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        use Error::*;
        match self {
            GetRootOnEmpty => write!(f, "Get root on an empty MMR")?,
            InconsistentlyStored(num) => write!(f, "Inconsistent store {}", num)?,
            StoreError(msg) => write!(f, "Store error {}", msg)?,
            CorruptedProof => write!(f, "Corrupted proof")?,
            GenProofForInvalidLeaves => write!(f, "Generate proof ofr invalid leaves")?,
        }
        Ok(())
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        impl ::std::error::Error for Error {}
    }
}
