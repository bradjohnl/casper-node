//! Support for runtime configuration of the execution engine - as an integral property of the
//! `EngineState` instance.
mod fee_handling;
mod refund_handling;

use std::collections::BTreeSet;

use num_rational::Ratio;
use num_traits::One;

use casper_types::{account::AccountHash, PublicKey};

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
/// Default value for allowing unrestricted transfers.
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
    pub(crate) administrative_accounts: BTreeSet<AccountHash>,
    /// Auction entrypoints such as "add_bid" or "delegate" are disabled if this flag is set to
    /// `true`.
    pub(crate) allow_auction_bids: bool,
    /// Allow unrestricted transfers between normal accounts.
    ///
    /// If set to `true` accounts can transfer tokens between themselves without restrictions. If
    /// set to `false` tokens can be transferred only from normal accounts to administrators
    /// and administrators to normal accounts but not normal accounts to normal accounts.
    pub(crate) allow_unrestricted_transfers: bool,
    /// Refund handling config.
    pub(crate) refund_handling: RefundHandling,
    /// Fee handling.
    pub(crate) fee_handling: FeeHandling,
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
    ///
    /// New code should use [`crate::core::engine_state::EngineConfigBuilder`] instead as some of
    /// the newer config flags might be defaulted.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        max_query_depth: u64,
        max_associated_keys: u32,
        max_runtime_call_stack_height: u32,
        minimum_delegation_amount: u64,
        strict_argument_checking: bool,
        wasm_config: WasmConfig,
        system_config: SystemConfig,
    ) -> EngineConfig {
        Self {
            max_query_depth,
            max_associated_keys,
            max_runtime_call_stack_height,
            minimum_delegation_amount,
            strict_argument_checking,
            wasm_config,
            system_config,
            administrative_accounts: Default::default(),
            allow_auction_bids: DEFAULT_ALLOW_AUCTION_BIDS,
            allow_unrestricted_transfers: DEFAULT_ALLOW_UNRESTRICTED_TRANSFERS,
            refund_handling: DEFAULT_REFUND_HANDLING,
            fee_handling: DEFAULT_FEE_HANDLING,
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

    /// Get the engine config's administrative accounts.
    pub fn administrative_accounts(&self) -> &BTreeSet<AccountHash> {
        &self.administrative_accounts
    }

    /// Get the engine config's allow auction bids.
    pub fn allow_auction_bids(&self) -> bool {
        self.allow_auction_bids
    }

    /// Get the engine config's allow unrestricted transfers.
    pub fn allow_unrestricted_transfers(&self) -> bool {
        self.allow_unrestricted_transfers
    }

    /// Checks if an account hash is an administrator.
    ///
    /// This method returns a `None` if there is no administrators configured.
    /// Otherwise returns Some with a flag indicating if a passed account hash is an admin.
    pub(crate) fn is_account_administrator(&self, account_hash: &AccountHash) -> Option<bool> {
        // Ensure there's at least one administrator configured.
        if self.administrative_accounts.is_empty() {
            return None;
        }

        if account_hash == &PublicKey::System.to_account_hash() {
            return Some(true);
        }

        // Find an administrator by its account hash.
        Some(self.administrative_accounts.contains(account_hash))
    }

    /// Get the engine config's refund ratio.
    pub fn refund_handling(&self) -> &RefundHandling {
        &self.refund_handling
    }

    /// Get the engine config's fee handling.
    pub fn fee_handling(&self) -> FeeHandling {
        self.fee_handling
    }
}

/// This is a builder pattern applied to the [`EngineConfig`] structure to shield any changes to the
/// constructor, or contents of it from the rest of the system.
///
/// Any field that isn't specified will be defaulted.
#[derive(Default, Debug)]
pub struct EngineConfigBuilder {
    max_query_depth: Option<u64>,
    max_associated_keys: Option<u32>,
    max_runtime_call_stack_height: Option<u32>,
    wasm_config: Option<WasmConfig>,
    system_config: Option<SystemConfig>,
    minimum_delegation_amount: Option<u64>,
    strict_argument_checking: Option<bool>,
    administrative_accounts: Option<BTreeSet<PublicKey>>,
    allow_auction_bids: Option<bool>,
    allow_unrestricted_transfers: Option<bool>,
    refund_handling: Option<RefundHandling>,
    fee_handling: Option<FeeHandling>,
}

impl EngineConfigBuilder {
    /// Create new `EngineConfig` builder object.
    pub fn new() -> Self {
        EngineConfigBuilder::default()
    }

    /// Set a max query depth config option.
    pub fn with_max_query_depth(mut self, max_query_depth: u64) -> Self {
        self.max_query_depth = Some(max_query_depth);
        self
    }

    /// Set a max associated keys config option.
    pub fn with_max_associated_keys(mut self, max_associated_keys: u32) -> Self {
        self.max_associated_keys = Some(max_associated_keys);
        self
    }

    /// Set a max runtime call stack height option.
    pub fn with_max_runtime_call_stack_height(
        mut self,
        max_runtime_call_stack_height: u32,
    ) -> Self {
        self.max_runtime_call_stack_height = Some(max_runtime_call_stack_height);
        self
    }

    /// Set a new wasm config configuration option.
    pub fn with_wasm_config(mut self, wasm_config: WasmConfig) -> Self {
        self.wasm_config = Some(wasm_config);
        self
    }

    /// Set a new system config configuration option.
    pub fn with_system_config(mut self, system_config: SystemConfig) -> Self {
        self.system_config = Some(system_config);
        self
    }

    /// Sets new maximum wasm memory.
    pub fn with_wasm_max_stack_height(mut self, wasm_stack_height: u32) -> Self {
        let wasm_config = self.wasm_config.get_or_insert_with(WasmConfig::default);
        wasm_config.max_stack_height = wasm_stack_height;
        self
    }

    /// Set a new system config configuration option.
    pub fn with_minimum_delegation_amount(mut self, minimum_delegation_amount: u64) -> Self {
        self.minimum_delegation_amount = Some(minimum_delegation_amount);
        self
    }

    /// Sets new maximum wasm memory.
    pub fn with_strict_argument_checking(mut self, strict_argument_checking: bool) -> Self {
        self.strict_argument_checking = Some(strict_argument_checking);
        self
    }

    /// Sets new chain kind.
    pub fn with_administrative_accounts(
        mut self,
        administrator_accounts: BTreeSet<PublicKey>,
    ) -> Self {
        self.administrative_accounts = Some(administrator_accounts);
        self
    }

    /// Sets new disable auction bids flag.
    pub fn with_allow_auction_bids(mut self, allow_auction_bids: bool) -> Self {
        self.allow_auction_bids = Some(allow_auction_bids);
        self
    }

    /// Set the engine config builder's allow unrestricted transfers.
    pub fn with_allow_unrestricted_transfers(mut self, allow_unrestricted_transfers: bool) -> Self {
        self.allow_unrestricted_transfers = Some(allow_unrestricted_transfers);
        self
    }

    /// Set the engine config builder's refund handling.
    pub fn with_refund_handling(mut self, refund_handling: RefundHandling) -> Self {
        match refund_handling {
            RefundHandling::Refund { refund_ratio } => {
                debug_assert!(
                    refund_ratio <= Ratio::one(),
                    "refund ratio should be a proper fraction"
                );
            }
        }

        self.refund_handling = Some(refund_handling);
        self
    }

    /// Set the engine config builder's fee handling.
    pub fn with_fee_handling(mut self, fee_handling: FeeHandling) -> Self {
        self.fee_handling = Some(fee_handling);
        self
    }

    /// Build a new [`EngineConfig`] object.
    pub fn build(self) -> EngineConfig {
        let max_query_depth = self.max_query_depth.unwrap_or(DEFAULT_MAX_QUERY_DEPTH);
        let max_associated_keys = self
            .max_associated_keys
            .unwrap_or(DEFAULT_MAX_ASSOCIATED_KEYS);
        let max_runtime_call_stack_height = self
            .max_runtime_call_stack_height
            .unwrap_or(DEFAULT_MAX_RUNTIME_CALL_STACK_HEIGHT);
        let minimum_delegation_amount = self
            .minimum_delegation_amount
            .unwrap_or(DEFAULT_MINIMUM_DELEGATION_AMOUNT);
        let strict_argument_checking = self
            .strict_argument_checking
            .unwrap_or(DEFAULT_STRICT_ARGUMENT_CHECKING);
        let wasm_config = self.wasm_config.unwrap_or_default();
        let system_config = self.system_config.unwrap_or_default();
        let administrative_accounts = {
            self.administrative_accounts
                .unwrap_or_default()
                .iter()
                .map(PublicKey::to_account_hash)
                .collect()
        };
        let allow_auction_bids = self
            .allow_auction_bids
            .unwrap_or(DEFAULT_ALLOW_AUCTION_BIDS);
        let allow_unrestricted_transfers = self
            .allow_unrestricted_transfers
            .unwrap_or(DEFAULT_ALLOW_UNRESTRICTED_TRANSFERS);
        let refund_handling = self.refund_handling.unwrap_or(DEFAULT_REFUND_HANDLING);
        let fee_handling = self.fee_handling.unwrap_or(DEFAULT_FEE_HANDLING);

        EngineConfig {
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
}
