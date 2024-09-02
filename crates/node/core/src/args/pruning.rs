//! Pruning and full node arguments

use crate::args::error::ReceiptsLogError;
use clap::Args;
use reth_chainspec::ChainSpec;
use reth_config::config::PruneConfig;
use reth_primitives::{Address, BlockNumber};
use reth_prune_types::{PruneMode, PruneModes, ReceiptsLogPruneConfig, MINIMUM_PRUNING_DISTANCE};
use std::collections::BTreeMap;

/// Parameters for pruning and full node
#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
#[command(next_help_heading = "Pruning")]
pub struct PruningArgs {
    /// Run full node. Only the most recent [`MINIMUM_PRUNING_DISTANCE`] block states are stored.
    #[arg(long, default_value_t = false)]
    pub full: bool,

    /// Minimum pruning interval measured in blocks.
    #[arg(long, default_value_t = 0)]
    pub block_interval: u64,

    // Sender Recovery
    /// Prunes all sender recovery data.
    #[arg(long = "prune.senderrecovery.full", conflicts_with_all = &["sender_recovery_distance", "sender_recovery_before"])]
    pub sender_recovery_full: bool,
    /// Prune sender recovery data before the `head-N` block number. In other words, keep last N +
    /// 1 blocks.
    #[arg(long = "prune.senderrecovery.distance", value_name = "BLOCKS", conflicts_with_all = &["sender_recovery_full", "sender_recovery_before"])]
    pub sender_recovery_distance: Option<u64>,
    /// Prune sender recovery data before the specified block number. The specified block number is
    /// not pruned.
    #[arg(long = "prune.senderrecovery.before", value_name = "BLOCK_NUMBER", conflicts_with_all = &["sender_recovery_full", "sender_recovery_distance"])]
    pub sender_recovery_before: Option<BlockNumber>,

    // Transaction Lookup
    /// Prunes all transaction lookup data.
    #[arg(long = "prune.transactionlookup.full", conflicts_with_all = &["transaction_lookup_distance", "transaction_lookup_before"])]
    pub transaction_lookup_full: bool,
    /// Prune transaction lookup data before the `head-N` block number. In other words, keep last N
    /// + 1 blocks.
    #[arg(long = "prune.transactionlookup.distance", value_name = "BLOCKS", conflicts_with_all = &["transaction_lookup_full", "transaction_lookup_before"])]
    pub transaction_lookup_distance: Option<u64>,
    /// Prune transaction lookup data before the specified block number. The specified block number
    /// is not pruned.
    #[arg(long = "prune.transactionlookup.before", value_name = "BLOCK_NUMBER", conflicts_with_all = &["transaction_lookup_full", "transaction_lookup_distance"])]
    pub transaction_lookup_before: Option<BlockNumber>,

    // Receipts
    /// Prunes all receipt data.
    #[arg(long = "prune.receipts.full", conflicts_with_all = &["receipts_distance", "receipts_before"])]
    pub receipts_full: bool,
    /// Prune receipts before the `head-N` block number. In other words, keep last N + 1 blocks.
    #[arg(long = "prune.receipts.distance", value_name = "BLOCKS", conflicts_with_all = &["receipts_full", "receipts_before"])]
    pub receipts_distance: Option<u64>,
    /// Prune receipts before the specified block number. The specified block number is not pruned.
    #[arg(long = "prune.receipts.before", value_name = "BLOCK_NUMBER", conflicts_with_all = &["receipts_full", "receipts_distance"])]
    pub receipts_before: Option<BlockNumber>,

    // Account History
    /// Prunes all account history.
    #[arg(long = "prune.accounthistory.full", conflicts_with_all = &["account_history_distance", "account_history_before"])]
    pub account_history_full: bool,
    /// Prune account before the `head-N` block number. In other words, keep last N + 1 blocks.
    #[arg(long = "prune.accounthistory.distance", value_name = "BLOCKS", conflicts_with_all = &["account_history_full", "account_history_before"])]
    pub account_history_distance: Option<u64>,
    /// Prune account history before the specified block number. The specified block number is not
    /// pruned.
    #[arg(long = "prune.accounthistory.before", value_name = "BLOCK_NUMBER", conflicts_with_all = &["account_history_full", "account_history_distance"])]
    pub account_history_before: Option<BlockNumber>,

    // Storage History
    /// Prunes all storage history data.
    #[arg(long = "prune.storagehistory.full", conflicts_with_all = &["storage_history_distance", "storage_history_before"])]
    pub storage_history_full: bool,
    /// Prune storage history before the `head-N` block number. In other words, keep last N + 1
    /// blocks.
    #[arg(long = "prune.storagehistory.distance", value_name = "BLOCKS", conflicts_with_all = &["storage_history_full", "storage_history_before"])]
    pub storage_history_distance: Option<u64>,
    /// Prune storage history before the specified block number. The specified block number is not
    /// pruned.
    #[arg(long = "prune.storagehistory.before", value_name = "BLOCK_NUMBER", conflicts_with_all = &["storage_history_full", "storage_history_distance"])]
    pub storage_history_before: Option<BlockNumber>,

    // Receipts Log Filter
    /// Configure receipts log filter. Format:
    /// <`address`>:<`prune_mode`>[,<`address`>:<`prune_mode`>...] Where <`prune_mode`> can be
    /// 'full', 'distance:<`blocks`>', or 'before:<`block_number`>'
    #[arg(long = "prune.receiptslogfilter", value_name = "FILTER_CONFIG", value_delimiter = ',')]
    pub receipts_log_filter: Vec<String>,
}

impl PruningArgs {
    /// Returns pruning configuration.
    pub fn prune_config(&self, chain_spec: &ChainSpec) -> Option<PruneConfig> {
        // Initialise with a default prune configuration.
        let mut config = PruneConfig::default();

        // If --full is set, use full node defaults.
        if self.full {
            config = PruneConfig {
                block_interval: 5,
                segments: PruneModes {
                    sender_recovery: Some(PruneMode::Full),
                    transaction_lookup: None,
                    // prune all receipts if chain doesn't have deposit contract specified in chain
                    // spec
                    receipts: chain_spec
                        .deposit_contract
                        .as_ref()
                        .map(|contract| PruneMode::Before(contract.block))
                        .or(Some(PruneMode::Full)),
                    account_history: Some(PruneMode::Distance(MINIMUM_PRUNING_DISTANCE)),
                    storage_history: Some(PruneMode::Distance(MINIMUM_PRUNING_DISTANCE)),
                    receipts_log_filter: ReceiptsLogPruneConfig(
                        chain_spec
                            .deposit_contract
                            .as_ref()
                            .map(|contract| (contract.address, PruneMode::Before(contract.block)))
                            .into_iter()
                            .collect(),
                    ),
                },
            }
        }

        // Override with any explicitly set prune.* flags.
        if let Some(mode) = self.sender_recovery_prune_mode() {
            config.segments.sender_recovery = Some(mode);
        }
        if let Some(mode) = self.transaction_lookup_prune_mode() {
            config.segments.transaction_lookup = Some(mode);
        }
        if let Some(mode) = self.receipts_prune_mode() {
            config.segments.receipts = Some(mode);
        }
        if let Some(mode) = self.account_history_prune_mode() {
            config.segments.account_history = Some(mode);
        }
        if let Some(mode) = self.storage_history_prune_mode() {
            config.segments.storage_history = Some(mode);
        }

        if !self.receipts_log_filter.is_empty() {
            match self.parse_receipts_log_filter() {
                Ok(filter_config) => config.segments.receipts_log_filter = filter_config,
                Err(e) => {
                    eprintln!("Error parsing receipts log filter: {}", e);
                }
            }
        }

        Some(config)
    }
    const fn sender_recovery_prune_mode(&self) -> Option<PruneMode> {
        if self.sender_recovery_full {
            Some(PruneMode::Full)
        } else if let Some(distance) = self.sender_recovery_distance {
            Some(PruneMode::Distance(distance))
        } else if let Some(block_number) = self.sender_recovery_before {
            Some(PruneMode::Before(block_number))
        } else {
            None
        }
    }

    const fn transaction_lookup_prune_mode(&self) -> Option<PruneMode> {
        if self.transaction_lookup_full {
            Some(PruneMode::Full)
        } else if let Some(distance) = self.transaction_lookup_distance {
            Some(PruneMode::Distance(distance))
        } else if let Some(block_number) = self.transaction_lookup_before {
            Some(PruneMode::Before(block_number))
        } else {
            None
        }
    }

    const fn receipts_prune_mode(&self) -> Option<PruneMode> {
        if self.receipts_full {
            Some(PruneMode::Full)
        } else if let Some(distance) = self.receipts_distance {
            Some(PruneMode::Distance(distance))
        } else if let Some(block_number) = self.receipts_before {
            Some(PruneMode::Before(block_number))
        } else {
            None
        }
    }

    const fn account_history_prune_mode(&self) -> Option<PruneMode> {
        if self.account_history_full {
            Some(PruneMode::Full)
        } else if let Some(distance) = self.account_history_distance {
            Some(PruneMode::Distance(distance))
        } else if let Some(block_number) = self.account_history_before {
            Some(PruneMode::Before(block_number))
        } else {
            None
        }
    }

    const fn storage_history_prune_mode(&self) -> Option<PruneMode> {
        if self.storage_history_full {
            Some(PruneMode::Full)
        } else if let Some(distance) = self.storage_history_distance {
            Some(PruneMode::Distance(distance))
        } else if let Some(block_number) = self.storage_history_before {
            Some(PruneMode::Before(block_number))
        } else {
            None
        }
    }

    fn parse_receipts_log_filter(&self) -> Result<ReceiptsLogPruneConfig, ReceiptsLogError> {
        let mut config = BTreeMap::new();
        // Each filter is provided in the form <address>:<prunemode>
        for filter in &self.receipts_log_filter {
            let parts: Vec<&str> = filter.split(':').collect();
            if parts.len() < 2 {
                return Err(ReceiptsLogError::InvalidFilterFormat(filter.clone()));
            }
            // Parse the address
            let address = parts[0]
                .parse::<Address>()
                .map_err(|_| ReceiptsLogError::InvalidAddress(parts[0].to_string()))?;

            // Parse the prune mode
            let prune_mode = match parts[1] {
                "full" => PruneMode::Full,
                s if s.starts_with("distance") => {
                    let distance =
                        parts[2].parse::<u64>().map_err(ReceiptsLogError::InvalidDistance)?;
                    PruneMode::Distance(distance)
                }
                s if s.starts_with("before") => {
                    let block_number = parts[2]
                        .parse::<BlockNumber>()
                        .map_err(ReceiptsLogError::InvalidBlockNumber)?;
                    PruneMode::Before(block_number)
                }
                _ => return Err(ReceiptsLogError::InvalidPruneMode(parts[0].to_string())),
            };
            config.insert(address, prune_mode);
        }
        Ok(ReceiptsLogPruneConfig(config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// A helper type to parse Args more easily
    #[derive(Parser)]
    struct CommandParser<T: Args> {
        #[command(flatten)]
        args: T,
    }

    /// Helper function for constructing `PruningArgs` with a `ReceiptsLogFilter`
    fn create_pruning_args(filters: Vec<String>) -> PruningArgs {
        PruningArgs { receipts_log_filter: filters, ..Default::default() }
    }

    #[test]
    fn pruning_args_sanity_check() {
        let default_args = PruningArgs::default();
        let args = CommandParser::<PruningArgs>::parse_from(["reth"]).args;
        assert_eq!(args, default_args);
    }

    #[test]
    fn test_parse_receipts_log_filter() {
        let addr1 = "0x0000000000000000000000000000000000000001";
        let addr2 = "0x0000000000000000000000000000000000000002";
        let addr3 = "0x0000000000000000000000000000000000000003";
        let args = create_pruning_args(vec![
            format!("{}:full", addr1),
            format!("{}:distance:1000", addr2),
            format!("{}:before:5000000", addr3),
        ]);

        // Args can be parsed.
        let result = args.parse_receipts_log_filter();
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.0.len(), 3);

        // Check that the args were parsed correctly.
        let addr1: Address = addr1.parse().unwrap();
        let addr2: Address = addr2.parse().unwrap();
        let addr3: Address = addr3.parse().unwrap();

        assert_eq!(config.0.get(&addr1), Some(&PruneMode::Full));
        assert_eq!(config.0.get(&addr2), Some(&PruneMode::Distance(1000)));
        assert_eq!(config.0.get(&addr3), Some(&PruneMode::Before(5000000)));
    }

    #[test]
    fn test_parse_receipts_log_filter_invalid_filter_format() {
        let args = create_pruning_args(vec!["invalid_format".to_string()]);
        let result = args.parse_receipts_log_filter();
        assert!(matches!(result, Err(ReceiptsLogError::InvalidFilterFormat(_))));
    }

    #[test]
    fn test_parse_receipts_log_filter_invalid_address() {
        let args = create_pruning_args(vec!["invalid_address:full".to_string()]);
        let result = args.parse_receipts_log_filter();
        assert!(matches!(result, Err(ReceiptsLogError::InvalidAddress(_))));
    }

    #[test]
    fn test_parse_receipts_log_filter_invalid_prune_mode() {
        let args = create_pruning_args(vec![
            "0x0000000000000000000000000000000000000000:invalid_mode".to_string(),
        ]);
        let result = args.parse_receipts_log_filter();
        assert!(matches!(result, Err(ReceiptsLogError::InvalidPruneMode(_))));
    }

    #[test]
    fn test_parse_receipts_log_filter_invalid_distance() {
        let args = create_pruning_args(vec![
            "0x0000000000000000000000000000000000000000:distance:invalid_distance".to_string(),
        ]);
        let result = args.parse_receipts_log_filter();
        assert!(matches!(result, Err(ReceiptsLogError::InvalidDistance(_))));
    }

    #[test]
    fn test_parse_receipts_log_filter_invalid_block_number() {
        let args = create_pruning_args(vec![
            "0x0000000000000000000000000000000000000000:before:invalid_block".to_string(),
        ]);
        let result = args.parse_receipts_log_filter();
        assert!(matches!(result, Err(ReceiptsLogError::InvalidBlockNumber(_))));
    }
}
