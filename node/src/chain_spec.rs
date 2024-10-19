use hex_literal::hex;
use sc_service::{ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
// use sp_runtime::key_types::IM_ONLINE;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_core::crypto::UncheckedInto;

use global_network_runtime::{
	currency::*, opaque::SessionKeys, AccountId, AuraConfig, BalancesConfig,
	CouncilConfig, DemocracyConfig, GenesisConfig, GrandpaConfig,
	ImOnlineConfig, SessionConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig,
	ValidatorSetConfig, WASM_BINARY,
};
use std::{ default::Default};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

// type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate a crypto pair from seed
fn get_from_secret<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(seed, None)
		.unwrap_or_else(|_| panic!("Invalid string '{}'", seed))
		.public()
}

const ALITH: &str = "0x2FBAC9dE90e988fB2014FC8Aa08cf452e5E5E515";
const BALTATHAR: &str = "0xF3c25Ea246B52a901b47CDAE1ecD3039246Ab31d";
const CHARLETH: &str = "0xac0103172516afe69E9F3D3EB451cb6382b3A0EB";
const DOROTHY: &str = "0x2651E1424Fa908982eBAA6aaCdf899E8783B028f";

pub fn public_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		"Global Foundation Mainnet",
		"public_live",
		ChainType::Live,
		move || {
			mainnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					(
						array_bytes::hex_n_into_unchecked(ALITH),
						hex!["469af7baae9f43aa9eed5db7c13c25474d299093934dc0d112c31e26935d7f12"].unchecked_into(),
						hex!["baabfa2e04240e1a0496181f09d2bf6301d270b4945afe3ccef88d3e3957096f"].unchecked_into(),
						hex!["78c8c415434d9f07eb8c005ad63cc9c9510a3dc9037315494168c2a9bc50f870"].unchecked_into(),
					),
					(
						array_bytes::hex_n_into_unchecked(BALTATHAR),
						hex!["c02ed3d8dae2da54d510700cd2aecea0abd8bf474c954762655528d86af13b06"].unchecked_into(),
						hex!["58e59fab0f5d6a3c4692f4b602e85165286e4f09f15a673e4a1058a0c7426a76"].unchecked_into(),
						hex!["b282f91c01ad185d72986ec20d38b9e7b9b881aa00a0f0909f4e347b6d68f33c"].unchecked_into(),					),
					(	
						array_bytes::hex_n_into_unchecked(CHARLETH),
						hex!["0269f072fdc7dd2fcb8c9d43f01824eab0003f66f5bba12607ca858e95814d0f"].unchecked_into(),
						hex!["f3edd6930998935bc773ead7788062afa5a4521e060ede6a8be8b0ec642a2cc7"].unchecked_into(),
						hex!["926b059ae0259fd5caad7dc8433748acff41fbd03e04d31c0562d047e3932500"].unchecked_into(),					),
					(
						array_bytes::hex_n_into_unchecked(DOROTHY),
						hex!["f88c4d10063dbac79b84ccd502d491947330dd08422a147b2afa5fb422235b13"].unchecked_into(),
						hex!["b1484b312b581926e816253ca6e84c38ed4c442a48ee1d4da4b83b32e199d3c7"].unchecked_into(),
						hex!["e0330096b62c3bf69f927625cfe2eedbeabd0f424dadaf1ac6739a7fd598cb74"].unchecked_into(),					),
				],
				// Sudo account
				array_bytes::hex_n_into_unchecked(ALITH),
				// Pre-funded accounts
				vec![
					array_bytes::hex_n_into_unchecked(ALITH),
					array_bytes::hex_n_into_unchecked(BALTATHAR),
					array_bytes::hex_n_into_unchecked(CHARLETH),
					array_bytes::hex_n_into_unchecked(DOROTHY),
				],
				true,
			)
		},
		vec![
			"/ip4/2.58.80.231/tcp/30333/p2p/12D3KooWQkRtjCWbdFdirGr76xfvB4W1WiJWW2ZCZzBBGvZWixQy".parse().unwrap(),
			"/ip4/2.58.80.230/tcp/30333/p2p/12D3KooWAWtiYVcEDB5QrEcMUC3FdwsM9bsDe4kqFDjcjY1KiuA6".parse().unwrap(),
			"/ip4/194.163.139.100/tcp/30333/p2p/12D3KooWChLxahZPDCrDiy5hBWeoy29GTQFQ3jnXpBCQpUr2QqeF".parse().unwrap(),
			"/ip4/2.58.80.229/tcp/30333/p2p/12D3KooWMeyNeHAawVLBPv4bjhuHdiVYP2mBgAc9oNGWZChaVH86".parse().unwrap(),
		],
		None,
		None,
		None,
		Some(chainspec_properties()),
		None,
	))
}

fn session_keys(aura: AuraId, grandpa: GrandpaId, im_online: ImOnlineId) -> SessionKeys {
	SessionKeys { aura, grandpa, im_online }
}

pub fn chainspec_properties() -> Properties {
	let mut properties = Properties::new();
	properties.insert("tokenDecimals".into(), 18.into());
	properties.insert("tokenSymbol".into(), "GNF".into());
	properties
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Names
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![(
					array_bytes::hex_n_into_unchecked(ALITH),
					get_from_secret::<AuraId>("//Alice"),
					get_from_secret::<GrandpaId>("//Alice"),
					get_from_secret::<ImOnlineId>("//Alice"),
				)],
				// Sudo account
				array_bytes::hex_n_into_unchecked(ALITH),
				// Pre-funded accounts
				vec![
					array_bytes::hex_n_into_unchecked(ALITH),
					array_bytes::hex_n_into_unchecked(BALTATHAR),
					array_bytes::hex_n_into_unchecked(CHARLETH),
					array_bytes::hex_n_into_unchecked(DOROTHY),				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		Some(chainspec_properties()),
		// Extensions
		None,
	))
}

pub fn testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"GlobalFoundation Testnet",
		// ID
		"GlobalFoundation",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![
					(
						array_bytes::hex_n_into_unchecked(ALITH),
						hex!["d0ddca2479244563462089ace77b02a321c0ebb477e5d58aadcf9c96550c6c6c"].unchecked_into(),
						hex!["0e5d8ae48c24170cf741147405423a21ba274d6bb92722f80b28035dd607ee0f"].unchecked_into(),
						hex!["bcae62f9a0032a4cc88aa1df2083d41baf8e42dfc4a1307d98d52eec6e6fed0c"].unchecked_into(),
					),
					(
						array_bytes::hex_n_into_unchecked(BALTATHAR),
						hex!["c49f37c8db3c61026050d8df64564f82c3617b407d338852745d43af0bed656a"].unchecked_into(),
						hex!["d5e3e025405a775a832eab1a07ab3806d532d54c3334b999591f295de4cdc06f"].unchecked_into(),
						hex!["9c6c000297311ff8e611de88a3c9f870afdb31cf2966f762d378b7adc0e2ff0c"].unchecked_into(),					),
					(	
						array_bytes::hex_n_into_unchecked(CHARLETH),
						hex!["a2a2444ab2a6464c8ed775037b5f78965e3ac946d1d496bd2be891bf0d6ac164"].unchecked_into(),
						hex!["820a3a71c721d6c25d3577c5b7c286b0d30bdb17b1f25ecc4a37526ea61ce330"].unchecked_into(),
						hex!["8642122d2105edd69a2f5d35dddcd98f11af7295627a6178798dbef69e7afd03"].unchecked_into(),					),
					(
						array_bytes::hex_n_into_unchecked(DOROTHY),
						hex!["3cbeec7addfc9d98537343da13aac5addb35c7e41d5aa8a52c0af08783987701"].unchecked_into(),
						hex!["9e0fe45cd81f78fb2e793a46d17f783d6c0e44ae04f736563c94647a1a9d3bd5"].unchecked_into(),
						hex!["aa3cd41bdb289e6e819c9b6ce3597b4c8875f6386bf8a185c68ac5be77225e43"].unchecked_into(),					),
				],
				// Sudo account
				array_bytes::hex_n_into_unchecked(ALITH),
				// Pre-funded accounts
				vec![
					array_bytes::hex_n_into_unchecked(ALITH),
					array_bytes::hex_n_into_unchecked(BALTATHAR),
					array_bytes::hex_n_into_unchecked(CHARLETH),
					array_bytes::hex_n_into_unchecked(DOROTHY),
				],
				true,
			)
		},
		// Bootnodes
		vec![
			"/ip4/128.199.5.56/tcp/30333/p2p/12D3KooWKN78A6J3qMQVqdeN24jvQTxbq2oc9KPYsU6UJP9zqAKw".parse().unwrap(),
			"/ip4/167.71.184.0/tcp/30333/p2p/12D3KooWSuqj8A8dhFdqNHuDy2GztYESDap9EKjQpbn8GsFzmE7M".parse().unwrap(),
			"/ip4/167.71.92.0/tcp/30333/p2p/12D3KooWQvHq6ALB1UT7LdygR464PcdwX84uH2WGLemcyQyhHQV7".parse().unwrap(),
			"/ip4/167.172.246.0/tcp/30333/p2p/12D3KooWDxNG3UD7T3QZ3CVNJ2LCMhpfTqFBMGkRCop8phKexqsA".parse().unwrap(),
		],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		Some(chainspec_properties()),
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId, ImOnlineId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	let num_endowed_accounts = endowed_accounts.len();
	GenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| {
					if k == array_bytes::hex_n_into_unchecked(BALTATHAR) {
						(k.clone(), 1_755_000_000 * GNF)
					} else if k == array_bytes::hex_n_into_unchecked(CHARLETH) {
						(k.clone(), 66_000_000 * GNF)
					} else if k == array_bytes::hex_n_into_unchecked(DOROTHY) {
						(k.clone(), 194_999_000 * GNF)
					} else if k == array_bytes::hex_n_into_unchecked(ALITH) {
						(k.clone(), 1000 * GNF)
					} else {
						(k.clone(), 0 * GNF)
					}
				})
				.collect(),
		},

		validator_set: ValidatorSetConfig {
			initial_validators: initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
		},

		democracy: DemocracyConfig::default(),
 
		council: CouncilConfig::default(),
 
		technical_committee: TechnicalCommitteeConfig {
			members: endowed_accounts
				.iter()
				.take((num_endowed_accounts + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		},
 
		// aura: Default::default(),
		aura: AuraConfig {

			authorities: vec![],
		},
		grandpa: GrandpaConfig {

			authorities: vec![],
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		im_online: ImOnlineConfig { keys: vec![] },
		treasury: Default::default(),
		transaction_payment: Default::default(),

		evm : Default::default(),

		session: SessionConfig {
			keys: initial_authorities
				.into_iter()
				.map(|(acc, aura, gran, im_online)| {	
					(
						acc.clone(), acc,
						session_keys(
							aura, gran, im_online,
						),
					
					)
				})
				.collect::<Vec<_>>(),
		},

		ethereum: Default::default(),
		base_fee: Default::default(),
		
	}
}

fn mainnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId, ImOnlineId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	let num_endowed_accounts = endowed_accounts.len();

	GenesisConfig {

		treasury: Default::default(),
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
		},

		balances: BalancesConfig {

			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| {
					// if k == AccountId::from("0x90E79DAc498b35096d4d86CEa4f2c3681b40F5C7"). {
					if k == array_bytes::hex_n_into_unchecked(BALTATHAR) {
						(k.clone(), 24_750_000 * GNF)
					} else if k == array_bytes::hex_n_into_unchecked(CHARLETH) {
						(k.clone(), 24_750_000 * GNF)
					} else if k == array_bytes::hex_n_into_unchecked(DOROTHY) {
						(k.clone(), 24_750_000 * GNF)
					} else if k == array_bytes::hex_n_into_unchecked(ALITH) {
						(k.clone(), 24_750_000 * GNF)
					} else {
						(k.clone(), 0 * GNF)
					}
				})
				.collect(),
		},

		validator_set: ValidatorSetConfig {
			initial_validators: initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
		},

		aura: AuraConfig {
			authorities: vec![],
		},
		grandpa: GrandpaConfig {
			authorities: vec![],	
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		im_online: ImOnlineConfig { keys: vec![] },

		democracy: DemocracyConfig::default(),

		council: CouncilConfig::default(),

		technical_committee: TechnicalCommitteeConfig {
			members: endowed_accounts
				.iter()
				.take(num_endowed_accounts  )
				.cloned()
				.collect(),
			phantom: Default::default(),
		},

		transaction_payment: Default::default(),
		evm : Default::default(),

		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(), 
						x.0.clone(), 
						session_keys(x.1.clone(), x.2.clone(), x.3.clone()))
				})
				.collect::<Vec<_>>(),
		},

		ethereum: Default::default(),
		base_fee: Default::default(),
	}
}
