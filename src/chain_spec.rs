use primitives::{Pair, Public};
use tiedye_runtime::{
	AccountId, BabeConfig, BalancesConfig, GenesisConfig, GrandpaConfig,
	SessionConfig, SessionKeys, StakerStatus, StakingConfig,
	SudoConfig, IndicesConfig, SystemConfig, WASM_BINARY, 
};
use babe_primitives::{AuthorityId as BabeId};
use grandpa_primitives::{AuthorityId as GrandpaId};
use tiedye_primitives::Balance;
use tiedye_runtime::oracle::sr25519::{AuthorityId as OracleId};
use substrate_service;
use sr_primitives::Perbill;

// Note this is the URL for the telemetry server
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
	/// Whatever the current runtime is, with just Alice as an auth.
	Development,
	/// Whatever the current runtime is, with simple Alice/Bob auths.
	LocalTestnet,
	/// Tiedye testnet v0.0.1
	Sunburst,
}

fn session_keys(grandpa: GrandpaId, babe: BabeId, oracle: OracleId) -> SessionKeys {
	SessionKeys { grandpa, babe, oracle }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, GrandpaId, BabeId, OracleId) {
	(
		get_from_seed::<AccountId>(&format!("{}//stash", seed)),
		get_from_seed::<AccountId>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<OracleId>(seed),
	)
}

impl Alternative {
	/// Get an actual chain config from one of the alternatives.
	pub(crate) fn load(self) -> Result<ChainSpec, String> {
		Ok(match self {
			Alternative::Development => ChainSpec::from_genesis(
				"Development",
				"dev",
				|| testnet_genesis(vec![
					get_authority_keys_from_seed("Alice"),
				],
				get_from_seed::<AccountId>("Alice"),
				vec![
					get_from_seed::<AccountId>("Alice"),
					get_from_seed::<AccountId>("Bob"),
					get_from_seed::<AccountId>("Alice//stash"),
					get_from_seed::<AccountId>("Bob//stash"),
				],
				true),
				vec![],
				None,
				None,
				None,
				None
			),
			Alternative::LocalTestnet => ChainSpec::from_genesis(
				"Local Testnet",
				"local_testnet",
				|| testnet_genesis(vec![
					get_authority_keys_from_seed("Alice"),
					get_authority_keys_from_seed("Bob"),
				], 
				get_from_seed::<AccountId>("Alice"),
				vec![
					get_from_seed::<AccountId>("Alice"),
					get_from_seed::<AccountId>("Bob"),
					get_from_seed::<AccountId>("Charlie"),
					get_from_seed::<AccountId>("Dave"),
					get_from_seed::<AccountId>("Eve"),
					get_from_seed::<AccountId>("Ferdie"),
					get_from_seed::<AccountId>("Alice//stash"),
					get_from_seed::<AccountId>("Bob//stash"),
					get_from_seed::<AccountId>("Charlie//stash"),
					get_from_seed::<AccountId>("Dave//stash"),
					get_from_seed::<AccountId>("Eve//stash"),
					get_from_seed::<AccountId>("Ferdie//stash"),
				],
				true),
				vec![],
				None,
				None,
				None,
				None
			),
			Alternative::Sunburst => ChainSpec::from_json_file(std::path::PathBuf::from("../sunburst.json")).unwrap(),
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"dev" => Some(Alternative::Development),
			"" | "local" => Some(Alternative::LocalTestnet),
			"sunburst" => Some(Alternative::Sunburst),
			_ => None,
		}
	}
}

const STASH: Balance = 1 << 59;

fn testnet_genesis(initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, OracleId)>,
	root_key: AccountId, 
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool) -> GenesisConfig {
	GenesisConfig {
		system: Some(SystemConfig {
			code: WASM_BINARY.to_vec(),
			changes_trie_config: Default::default(),
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.clone(),
		}),
		balances: Some(BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
			vesting: vec![],
		}),
		sudo: Some(SudoConfig {
			key: root_key,
		}),
		babe: Some(BabeConfig {
			.. Default::default()
		}),
		grandpa: Some(GrandpaConfig {
			.. Default::default()
		}),
		session: Some(SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(x.0.clone(), session_keys(x.2.clone(), x.3.clone(), x.4.clone()))
			}).collect::<Vec<_>>(),
		}),
		staking: Some(StakingConfig {
			current_era: 0,
			validator_count: 7,
			minimum_validator_count: 4,
			stakers: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.1.clone(), STASH, StakerStatus::Validator)
			}).collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(0),
			.. Default::default()
		}),
	}
}
