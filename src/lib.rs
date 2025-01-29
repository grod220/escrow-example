//! User escrows tokens and specifies a lockup period (release slot).
//!
//! Once lockup ends, you can take them out.
//!
//! User wallet         <s>    <s>      Escrow account
//! User token account  <------->       Escrow token account

pub mod entrypoint;
pub mod instruction;
pub mod processor;
pub mod state;
