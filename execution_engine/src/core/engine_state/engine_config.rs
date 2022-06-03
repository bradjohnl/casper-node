//! Support for runtime configuration of the execution engine - as an integral property of the
//! `EngineState` instance.
mod fee_handling;
mod refund_handling;

use std::collections::BTreeSet;

use num_rational::Ratio;

use casper_types::account::AccountHash;

use crate::shared::{system_config::SystemConfig, wasm_config::WasmConfig};

pub use self::{fee_handling::FeeHandling, refund_handling::RefundHandling};

/// Default value for a maximum query depth configuration option.
pub const DEFAULT_MAX_QUERY_DEPTH: u64 = 5;
/// Default value for maximum associated keys configuration option.
pub const DEFAULT_MAX_ASSOCIATED_KEYS: u32 = 100;
/// Default value for maximum runtime call stack height configuration option.
pub const DEFAULT_MAX_RUNTIME_CALL_STACK_HEIGHT: u32 = 12;
/// Default value for minimum delegation amount in motes.
pub const DEFAULT_MINIMUM_DELEGATION_AMOUNT: u64 = 500 * 1_000_000_000;
/// Default value for strict argument checking.
pub const DEFAULT_STRICT_ARGUMENT_CHECKING: bool = false;
/// Default value for allowing auction bids.
pub const DEFAULT_ALLOW_AUCTION_BIDS: bool = true;
/// Default value for allowing unrestricted transfers
pub const DEFAULT_ALLOW_UNRESTRICTED_TRANSFERS: bool = true;
/// Default gas cost refund ratio.
pub const DEFAULT_REFUND_HANDLING: RefundHandling = RefundHandling::Refund {
    refund_ratio: Ratio::new_raw(0, 100),
};
/// Default fee handling.
pub const DEFAULT_FEE_HANDLING: FeeHandling = FeeHandling::PayToProposer;

///
/// The runtime configuration of the execution engine
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Max query depth of the engine.
    pub(crate) max_query_depth: u64,
    /// Maximum number of associated keys (i.e. map of
    /// [`AccountHash`](casper_types::account::AccountHash)s to
    /// [`Weight`](casper_types::account::Weight)s) for a single account.
    max_associated_keys: u32,
    max_runtime_call_stack_height: u32,
    minimum_delegation_amount: u64,
    /// This flag indicates if arguments passed to contracts are checked against the defined types.
    strict_argument_checking: bool,
    wasm_config: WasmConfig,
    system_config: SystemConfig,
    /// A private network specifies a list of administrative accounts.
    administrative_accounts: BTreeSet<AccountHash>,
    /// Auction entrypoints such as "add_bid" or "delegate" are disabled if this flag is set to
    /// `true`.
    allow_auction_bids: bool,
    /// Allow unrestricted transfers between normal accounts.
    ///
    /// If set to `true` accounts can transfer tokens between themselves without restrictions. If
    /// set to `false` tokens can be transferred only from normal accounts to administrators
    /// and administrators to normal accounts but not normal accounts to normal accounts.
    allow_unrestricted_transfers: bool,
    /// Refund handling config.
    refund_handling: RefundHandling,
    /// Fee handling.
    fee_handling: FeeHandling,
}

impl Default for EngineConfig {
    fn default() -> Self {
        EngineConfig {
            max_query_depth: DEFAULT_MAX_QUERY_DEPTH,
            max_associated_keys: DEFAULT_MAX_ASSOCIATED_KEYS,
            max_runtime_call_stack_height: DEFAULT_MAX_RUNTIME_CALL_STACK_HEIGHT,
            minimum_delegation_amount: DEFAULT_MINIMUM_DELEGATION_AMOUNT,
            strict_argument_checking: DEFAULT_STRICT_ARGUMENT_CHECKING,
            wasm_config: WasmConfig::default(),
            system_config: SystemConfig::default(),
            administrative_accounts: Default::default(),
            allow_auction_bids: DEFAULT_ALLOW_AUCTION_BIDS,
            allow_unrestricted_transfers: DEFAULT_ALLOW_UNRESTRICTED_TRANSFERS,
            refund_handling: DEFAULT_REFUND_HANDLING,
            fee_handling: DEFAULT_FEE_HANDLING,
        }
    }
}

impl EngineConfig {
    /// Creates new [`EngineConfig`] instance.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        max_query_depth: u64,
        max_associated_keys: u32,
        max_runtime_call_stack_height: u32,
        minimum_delegation_amount: u64,
        strict_argument_checking: bool,
        wasm_config: WasmConfig,
        system_config: SystemConfig,
        administrative_accounts: BTreeSet<AccountHash>,
        allow_auction_bids: bool,
        allow_unrestricted_transfers: bool,
        refund_handling: RefundHandling,
        fee_handling: FeeHandling,
    ) -> Self {
        Self {
            max_query_depth,
            max_associated_keys,
            max_runtime_call_stack_height,
            minimum_delegation_amount,
            strict_argument_checking,
            wasm_config,
            system_config,
            administrative_accounts,
            allow_auction_bids,
            allow_unrestricted_transfers,
            refund_handling,
            fee_handling,
        }
    }

    /// Returns the current max associated keys config.
    pub fn max_associated_keys(&self) -> u32 {
        self.max_associated_keys
    }

    /// Returns the current max runtime call stack height config.
    pub fn max_runtime_call_stack_height(&self) -> u32 {
        self.max_runtime_call_stack_height
    }

    /// Returns the current wasm config.
    pub fn wasm_config(&self) -> &WasmConfig {
        &self.wasm_config
    }

    /// Returns the current system config.
    pub fn system_config(&self) -> &SystemConfig {
        &self.system_config
    }

    /// Returns the minimum delegation amount in motes.
    pub fn minimum_delegation_amount(&self) -> u64 {
        self.minimum_delegation_amount
    }

    /// Get the engine config's strict argument checking flag.
    pub fn strict_argument_checking(&self) -> bool {
        self.strict_argument_checking
    }

    /// Get the engine config's administrative accouAnts.
    #[must_use]
    pub fn administrative_accounts(&self) -> &BTreeSet<AccountHash> {
        &self.administrative_accounts
    }

    /// Get the engine config's allow auction bids.
    #[must_use]
    pub fn allow_auction_bids(&self) -> bool {
        self.allow_auction_bids
    }

    /// Get the engine config's allow unrestricted transfers.
    #[must_use]
    pub fn allow_unrestricted_transfers(&self) -> bool {
        self.allow_unrestricted_transfers
    }

    /// Checks if an account hash is an administrator.
    ///
    /// This method returns a `None` if there is no administrators configured.
    /// Otherwise returns Some with a flag indicating if a passed account hash is an admin.
    #[must_use]
    pub(crate) fn is_account_administrator(&self, account_hash: &AccountHash) -> Option<bool> {
        // Ensure there's at least one administrator configured.
        if self.administrative_accounts.is_empty() {
            return None;
        }

        // Find an administrator by its account hash.
        Some(self.administrative_accounts.contains(account_hash))
    }

    /// Get the engine config's refund ratio.
    #[must_use]
    pub fn refund_handling(&self) -> &RefundHandling {
        &self.refund_handling
    }

    /// Get the engine config's fee handling.
    #[must_use]
    pub fn fee_handling(&self) -> FeeHandling {
        self.fee_handling
    }
}
