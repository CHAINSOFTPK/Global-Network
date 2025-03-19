#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]


pub type AccountId = <<Signature as sp_runtime::traits::Verify>::Signer as sp_runtime::traits::IdentifyAccount>::AccountId;


// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));


use pallet_grandpa::{
	fg_primitives, AuthorityId as GrandpaId, AuthorityList as GrandpaAuthorityList,
};

use frame_system::EnsureRoot;


use codec::{Encode, Decode};
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata, U256, H160, H256};
use crate::currency::*;

use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys, generic::Era, 
	traits::{
		BlakeTwo256, Block as BlockT, NumberFor, One,
		Dispatchable, PostDispatchInfoOf, DispatchInfoOf, UniqueSaturatedInto, OpaqueKeys, 
		Verify 
	},
	transaction_validity::{
		TransactionSource,TransactionPriority, TransactionValidity, TransactionValidityError}, ApplyExtrinsicResult, ConsensusEngineId,
		SaturatedConversion,
};

use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use pallet_im_online::sr25519::AuthorityId as ImOnlineId;


use pallet_evm::{
	EnsureAddressRoot, EnsureAddressNever, Account as EVMAccount, Runner,
	FeeCalculator,
};

use pallet_ethereum::{Call::transact, EthereumBlockHashMapping, Transaction as EthereumTransaction};
use fp_rpc::TransactionStatus;
// pub use this so we can import it in the chain spec.
#[cfg(feature = "std")]
pub use fp_evm::GenesisAccount;


// A few exports that help ease life for downstream crates.
pub use frame_support::{
	construct_runtime, log, parameter_types, pallet_prelude::PhantomData,
	PalletId,
	traits::{
		ConstU128, ConstU32, ConstU64, ConstU8, KeyOwnerProofSystem, Randomness, StorageInfo,
		FindAuthor, OnUnbalanced, Currency, Imbalance, EitherOfDiverse, EqualPrivilegeOnly,
	},
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
		IdentityFee, Weight,
	},


	StorageValue,
};
pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::{ CurrencyAdapter, Multiplier};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};


/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = account::EthereumSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
// pub type AccountId = AccountId20;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Index = u32;

/// A hash of some data used by the chain.
pub type Hash = H256;

pub mod currency {
	use super::Balance;

	pub const SUPPLY_FACTOR: Balance = 100;

	pub const WEI: Balance = 1;
	pub const KILOWEI: Balance = 1_000;
	pub const MEGAWEI: Balance = 1_000_000;
	pub const GIGAWEI: Balance = 1_000_000_000;
	pub const MICROSTOR: Balance = 1_000_000_000_000;
	pub const MILLISTOR: Balance = 1_000_000_000_000_000;
	pub const GNF: Balance = 1_000_000_000_000_000_000;
	pub const KILOSTOR: Balance = 1_000_000_000_000_000_000_000;

	pub const TRANSACTION_BYTE_FEE: Balance = 1 * GIGAWEI * SUPPLY_FACTOR;
	pub const STORAGE_BYTE_FEE: Balance = 100 * MICROSTOR * SUPPLY_FACTOR;
	pub const WEIGHT_FEE: Balance = 50 * KILOWEI * SUPPLY_FACTOR;

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 100 * MILLISTOR * SUPPLY_FACTOR + (bytes as Balance) * STORAGE_BYTE_FEE
	}

	// The FC token with 18 decimals
	pub const FC: Balance = 1_000_000_000_000_000_000;  // 1 FC = 10^18 
	pub const KFC: Balance = 1_000 * FC;                // Thousand  
	pub const MFC: Balance = 1_000 * KFC;               // Million
	pub const BFC: Balance = 1_000 * MFC;               // Billion
}

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
			pub im_online: ImOnline,
		}
	}
}


parameter_types! {
	pub const MinAuthorities: u32 = 1;
}

impl validator_set::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type AddRemoveOrigin = EnsureRoot<AccountId>;
	type MinAuthorities = MinAuthorities;
	// type WeightInfo = validator_set::weights::SubstrateWeight<Runtime>;
}


// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("global-network"),
	impl_name: create_runtime_str!("global-network"),
	authoring_version: 1,
	// The version of the runtime specification. A full node will not attempt to use its native
	//   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	//   `spec_version`, and `authoring_version` are the same between Wasm and native.
	// This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
	//   the compatible custom types.
	spec_version: 102,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
	
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
pub const HOURS: BlockNumber = MINUTES * 60;
pub const DAYS: BlockNumber = HOURS * 24;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
// pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND.saturating_mul(2);
const WEIGHT_PER_GAS: u64 = 20_000;

mod precompiles;
mod account;

use precompiles::SubstratePrecompiles;

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::with_sensible_defaults(
			Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
			NORMAL_DISPATCH_RATIO,
		);
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
	
	// For FarmVentures chain ID - updated to 1204
	pub const FarmChainId: u64 = 2023;
}

// Function to determine which currency to use based on chain ID
pub fn get_native_currency() -> Balance {
	if <Runtime as pallet_evm::Config>::ChainId::get() == FarmChainId::get() {
		FC
	} else {
		GNF
	}
}

// Configure FRAME pallets to include in runtime.

impl frame_system::Config for Runtime {
	/// The basic call filter to use in dispatchable.
	type BaseCallFilter = frame_support::traits::Everything;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The ubiquitous origin type.
	type RuntimeOrigin = RuntimeOrigin;
	/// The aggregated dispatch type that is available for extrinsics.
	type RuntimeCall = RuntimeCall;
	/// The index type for storing how many extrinsics an account has signed.
	type Index = Index;
	/// The index type for blocks.
	type BlockNumber = BlockNumber;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// The hashing algorithm used.
	type Hashing = BlakeTwo256;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The lookup mechanism to get account ID from whatever is passed in dispatchers.
	// type Lookup = AccountIdLookup<AccountId, ()>;
	type Lookup = sp_runtime::traits::IdentityLookup<AccountId>;
	/// The header type.
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// Converts a module to the index of the module in `construct_runtime!`.
	///
	/// This type is being generated by `construct_runtime!`.
	type PalletInfo = PalletInfo;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// What to do if a new account is created.
	type OnNewAccount = ();
	/// What to do if an account is fully reaped from the system.
	type OnKilledAccount = ();
	/// Weight information for the extrinsics of this pallet.
	type SystemWeightInfo = ();
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	/// The set code logic, just the default since we're not a parachain.
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_randomness_collective_flip::Config for Runtime {}



impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type MaxAuthorities = ConstU32<32>;
	type DisabledValidators = ();
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type KeyOwnerProof =
	<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

	type KeyOwnerIdentification = <Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(
		KeyTypeId,
		GrandpaId,
	)>>::IdentificationTuple;

	type KeyOwnerProofSystem = ();

	type HandleEquivocation = ();

	type WeightInfo = ();
	type MaxAuthorities = ConstU32<32>;

}
 
impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

/// Existential deposit.
pub const EXISTENTIAL_DEPOSIT: u128 = 1 * currency::MICROSTOR;

impl pallet_balances::Config for Runtime {
	/// The type for recording an account's balance.
	type Balance = Balance;
	type DustRemoval = ();
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
}

parameter_types! {
	pub FeeMultiplier: Multiplier = Multiplier::one();
}

pub struct DealWithFees;
type NegativeImbalance = <Balances as Currency<AccountId>>::NegativeImbalance;
 
impl OnUnbalanced<NegativeImbalance> for DealWithFees {
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item=NegativeImbalance>) {
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 20% to treasury, 80% to author
			let mut split = fees.ration(20, 80);
			if let Some(tips) = fees_then_tips.next() {
				// for tips, if any, 100% (though this can be anything)
				// tips.merge_into(&mut split.1);
				tips.ration_merge_into(30, 70, &mut split);
			}
			Treasury::on_unbalanced(split.0);
			Author::on_unbalanced(split.1);
		}
	}
}

pub struct Author;
 
impl OnUnbalanced<NegativeImbalance> for Author {
	fn on_nonzero_unbalanced(amount: NegativeImbalance) {
		if let Some(author) = Authorship::author() {
			Balances::resolve_creating(&author, amount);
		}
	}
}
 

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = CurrencyAdapter<Balances, DealWithFees>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ();
}

 
 
impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
}


parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		BlockWeights::get().max_block;
	// Retry a scheduled item every 10 blocks (1 minute) until the preimage exists.
	pub const NoPreimagePostponement: Option<u32> = Some(10);
}




parameter_types! {
	pub const LeetChainId: u64 = 1013;
	pub BlockGasLimit: U256 = U256::from(NORMAL_DISPATCH_RATIO * WEIGHT_REF_TIME_PER_SECOND / WEIGHT_PER_GAS);
	pub PrecompilesValue: SubstratePrecompiles<Runtime> = SubstratePrecompiles::<_>::new();
}

pub struct FindAuthorTruncated<F>(PhantomData<F>);

impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorTruncated<F> {
	fn find_author<'a, I>(digests: I) -> Option<H160>
		where
			I: 'a + IntoIterator<Item=(ConsensusEngineId, &'a [u8])>,
	{
		use sp_core::crypto::ByteArray;
		F::find_author(digests).and_then(|i| {
			Aura::authorities().get(i as usize).and_then(|id| {
				let raw = id.to_raw_vec();

				if raw.len() >= 24 {
					Some(H160::from_slice(&raw[4..24]))
				} else {
					None
				}
			})
		})
	}
}

pub struct StorageFindAuthor<Inner>(PhantomData<Inner>);
impl<Inner> FindAuthor<H160> for StorageFindAuthor<Inner>
where
	Inner: FindAuthor<AccountId>,
{
	fn find_author<'a, I>(digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		Inner::find_author(digests).map(Into::into)
	}
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = BaseFee;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;

	type WeightPerGas = ();
	type BlockHashMapping = EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressRoot<AccountId>;d
	type WithdrawOrigin = EnsureAddressNever<AccountId>;

	type AddressMapping = account::IntoAddressMapping;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;

	type PrecompilesType = SubstratePrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = LeetChainId; // Use FarmChainId instead of LeetChainId
	type BlockGasLimit = BlockGasLimit;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	type OnChargeTransaction = ();
	type OnCreate = ();
	type FindAuthor = StorageFindAuthor<pallet_session::FindAccountFromAuthorIndex<Self, Aura>>;
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type StateRoot = pallet_ethereum::IntermediateStateRoot<Self>;
}

parameter_types! {
	pub DefaultElasticity: Permill = Permill::zero();
	pub DefaultBaseFeePerGas: U256 = U256::from(1_000_000_000);
}

pub struct BaseFeeThreshold;

impl pallet_base_fee::BaseFeeThreshold for BaseFeeThreshold {
	fn lower() -> Permill {
		Permill::zero()
	}
	fn ideal() -> Permill {
		Permill::from_parts(500_000)
	}
	fn upper() -> Permill {
		Permill::from_parts(1_000_000)
	}
}

impl pallet_base_fee::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Threshold = BaseFeeThreshold;
	// type IsActive = IsActive;
	type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
	type DefaultElasticity = DefaultElasticity;
}

parameter_types! {
	pub const UncleGenerations: BlockNumber = 0;
}


// Impleminting the frame system off-chain
// ////////////// // //////////////// //////////////// //////////////// //////////////// //////////////// //////////////// //////////////// //////////////// //////////////// //////////////

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		public: <Signature as Verify>::Signer,
		account: AccountId,
		nonce: Index,
	) -> Option<(RuntimeCall, <UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload)> {
		let tip = 0;
		let period =
			BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
		let current_block = System::block_number().saturated_into::<u64>().saturating_sub(1);
		let era = Era::mortal(period, current_block);
		let extra = (
			frame_system::CheckNonZeroSender::<Runtime>::new(),
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(era),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
		);

		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				log::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = account;
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (address, signature.into(), extra)))
	}
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	RuntimeCall: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = RuntimeCall;
}
 
impl pallet_im_online::Config for Runtime {
	type AuthorityId = ImOnlineId;
	// type Event = Event;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	type ValidatorSet = ValidatorSet;
	type ReportUnresponsiveness = ValidatorSet;
	type UnsignedPriority = ImOnlineUnsignedPriority;
	type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
	type MaxKeys = MaxKeys;
	type MaxPeerInHeartbeats = MaxPeerInHeartbeats;
	type MaxPeerDataEncodingSize = MaxPeerDataEncodingSize;
	type RuntimeEvent = RuntimeEvent;

}


impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Aura>;
	type UncleGenerations = UncleGenerations;
	type FilterUncle = ();
	type EventHandler = ();
}



parameter_types! {// Updated to FC
	pub const Period: u32 = 60 * MINUTES;
	pub const Offset: u32 = 0;
}


parameter_types! {

	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 1 * GNF;
	pub const ImOnlineUnsignedPriority: TransactionPriority = TransactionPriority::max_value();
	pub const SpendPeriod: BlockNumber = 1 * DAYS;
	pub const Burn: Permill = Permill::from_percent(50);
	pub const MaxApprovals: u32 = 100;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const MaxKeys: u32 = 10_000;
	pub const MaxPeerInHeartbeats: u32 = 10_000;
	pub const MaxPeerDataEncodingSize: u32 = 1_000;
}

impl pallet_treasury::Config for Runtime {
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<u128>;
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type ApproveOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 5>,
	>;
	type RejectOrigin = EitherOfDiverse<
		EnsureRoot<AccountId>,
		pallet_collective::EnsureProportionMoreThan<AccountId, CouncilCollective, 1, 2>,
	>;
	type RuntimeEvent = RuntimeEvent;
	type OnSlash = ();
	type ProposalBond = ProposalBond;
	type ProposalBondMinimum = ProposalBondMinimum;
	type ProposalBondMaximum = ();
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = ();
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type MaxApprovals = MaxApprovals;
	
}

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
}

parameter_types! {
	pub const TechnicalMotionDuration: BlockNumber = 5 * DAYS;
	pub const TechnicalMaxProposals: u32 = 100;// Updated to FC
	pub const TechnicalMaxMembers: u32 = 100;
	pub const CooloffPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const MaxProposals: u32 = 100;
}
parameter_types! {
	pub const LaunchPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const VotingPeriod: BlockNumber = 28 * 24 * 60 * MINUTES;
	pub const FastTrackVotingPeriod: BlockNumber = 3 * 24 * 60 * MINUTES;
type CouncilCollective = pallet_collective::Instance1;
	pub const EnactmentPeriod: BlockNumber = 30 * 24 * 60 * MINUTES;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}mpl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
type TechnicalCollective = pallet_collective::Instance2;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
impl pallet_collective::Config<TechnicalCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;;
	type Proposal = RuntimeCall;llective::PrimeDefaultVote;
	type RuntimeEvent = RuntimeEvent;ve::weights::SubstrateWeight<Runtime>;
	type MotionDuration = TechnicalMotionDuration;
	type MaxProposals = TechnicalMaxProposals;
	type MaxMembers = TechnicalMaxMembers;ctive::Instance2;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
}mpl pallet_collective::Config<TechnicalCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
impl pallet_democracy::Config for Runtime {
	// type Proposal = RuntimeCall;t;
	type RuntimeEvent = RuntimeEvent;tionDuration;
	type Currency = Balances;icalMaxProposals;
	type EnactmentPeriod = EnactmentPeriod;
	type LaunchPeriod = LaunchPeriod;ive::PrimeDefaultVote;
	type VotingPeriod = VotingPeriod;ve::weights::SubstrateWeight<Runtime>;
	type VoteLockingPeriod = EnactmentPeriod;
	type Scheduler = Scheduler;
	// Same as EnactmentPeriodig for Runtime {
	type MinimumDeposit = MinimumDeposit;
	/// A straight majority of the council can decide what their next motion is.
	type ExternalOrigin =ces;
	pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 2>;
	/// A super-majority can have the next scheduled referendum be a straight majority-carries vote.
	type ExternalMajorityOrigin =iod;
	pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>;
	/// A unanimous council can have the next scheduled referendum be a straight default-carries
	/// (NTB) vote.tmentPeriod
	type ExternalDefaultOrigin =mDeposit;
	pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 1, 1>;
	/// Two thirds of the technical committee can have an ExternalMajority/ExternalDefault vote
	/// be tabled immediately and with a shorter voting/enactment period.ve, 1, 2>;
	type FastTrackOrigin = n have the next scheduled referendum be a straight majority-carries vote.
	pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 2, 3>;
	type InstantOrigin =nsureProportionAtLeast<AccountId, CouncilCollective, 3, 4>;
	pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>;ult-carries
	type InstantAllowed = frame_support::traits::ConstBool<true>;
	type FastTrackVotingPeriod = FastTrackVotingPeriod;
	// To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	type CancellationOrigin =hnical committee can have an ExternalMajority/ExternalDefault vote
	pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
	type BlacklistOrigin = EnsureRoot<AccountId>;
	// To cancel a proposal before it has been passed, the technical committee must be unanimous or
	// Root must agree.=
	type CancelProposalOrigin = EitherOfDiverse<ccountId, TechnicalCollective, 1, 1>;
		EnsureRoot<AccountId>,	ame_support::traits::ConstBool<true>;
		pallet_collective::EnsureProportionAtLeast<AccountId, TechnicalCollective, 1, 1>,
	>; To cancel a proposal which has been passed, 2/3 of the council must agree to it.
	// Any single technical committee member may veto a coming council proposal, however they can
	// only do it once and it lasts only for the cool-off period.Collective, 2, 3>;
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type CooloffPeriod = CooloffPeriod;as been passed, the technical committee must be unanimous or
	// type PreimageByteDeposit = PreimageByteDeposit;
	// type OperationalPreimageOrigin = pallet_collective::EnsureMember<AccountId, CouncilCollective>;
	type Slash = Treasury;		
	type MaxDeposits = ConstU32<100>;ionAtLeast<AccountId, TechnicalCollective, 1, 1>,
	>;
	type Preimages = Preimage;mmittee member may veto a coming council proposal, however they can
	type MaxBlacklisted = ConstU32<100>; for the cool-off period.
	type VetoOrigin = pallet_collective::EnsureMember<AccountId, TechnicalCollective>;
	type PalletsOrigin = OriginCaller;;
	type MaxVotes = frame_support::traits::ConstU32<100>;
	type WeightInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;countId, CouncilCollective>;
	type MaxProposals = MaxProposals;
}type MaxDeposits = ConstU32<100>;

	type Preimages = Preimage;
/// Special `ValidatorIdOf` implementation that is just returning the input as result.
pub struct ValidatorIdOf;
impl sp_runtime::traits::Convert<AccountId, Option<AccountId>> for ValidatorIdOf {
	fn convert(a: AccountId) -> Option<AccountId> {<100>;
		Some(a)ghtInfo = pallet_democracy::weights::SubstrateWeight<Runtime>;
	}ype MaxProposals = MaxProposals;
}


impl pallet_session::Config for Runtime {n that is just returning the input as result.
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId =<Self as frame_system::Config>::AccountId;> for ValidatorIdOf {
	// type ValidatorIdOf = account::IdentityCollator;
	type ValidatorIdOf = validator_set::ValidatorOf<Self>;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
	// to be updated
	type SessionManager = ValidatorSet; 
	type SessionHandler = <opaque::SessionKeys as OpaqueKeys>::KeyTypeIdProviders;
	type Keys = opaque::SessionKeys;;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}// type ValidatorIdOf = account::IdentityCollator;
	type ValidatorIdOf = validator_set::ValidatorOf<Self>;
	type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
	type NextSessionRotation = pallet_session::PeriodicSessions<Period, Offset>;
// impl pallet_session::historical::Config for Runtime {
// 	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
// 	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;dProviders;
// }e Keys = opaque::SessionKeys;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
impl pallet_scheduler::Config for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeEvent = RuntimeEvent;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;al::Config for Runtime {
	type MaximumWeight = MaximumSchedulerWeight;:Exposure<AccountId, Balance>;
	type ScheduleOrigin = EnsureRoot<AccountId>;g::ExposureOf<Runtime>;
	type MaxScheduledPerBlock = ConstU32<50>;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = Preimage;meOrigin;
}type RuntimeEvent = RuntimeEvent;
	type PalletsOrigin = OriginCaller;
parameter_types! {= RuntimeCall;
	pub const PreimageMaxSize: u32 = 4096 * 1024;
	pub const PreimageBaseDeposit: Balance = 1 * FC; // Updated to FC
	// One cent: $10,000 / MB = ConstU32<50>;
	pub const PreimageByteDeposit: Balance = 1 * MILLISTOR;eight<Runtime>;
}type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = Preimage;
impl pallet_preimage::Config for Runtime {
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;: u32 = 4096 * 1024;
	type ManagerOrigin = EnsureRoot<AccountId>;* GNF;
	// type MaxSize = PreimageMaxSize;
	type BaseDeposit = PreimageBaseDeposit;= 1 * MILLISTOR;
	type ByteDeposit = PreimageByteDeposit;
}
impl pallet_preimage::Config for Runtime {
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
	type RuntimeEvent = RuntimeEvent;
// Create the runtime by composing the FRAME pallets that were previously configured.
construct_runtime!( = EnsureRoot<AccountId>;
	pub struct RuntimePreimageMaxSize;
	whereBaseDeposit = PreimageBaseDeposit;
		Block = Block,t = PreimageByteDeposit;
		NodeBlock = opaque::Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system,
		RandomnessCollectiveFlip: pallet_randomness_collective_flip, previously configured.
		Timestamp: pallet_timestamp,
	pub struct Runtime
		Balances: pallet_balances,
		TransactionPayment: pallet_transaction_payment,
		NodeBlock = opaque::Block,
		ValidatorSet: validator_set,edExtrinsic,
		Authorship: pallet_authorship,
		Session: pallet_session,
		Aura: pallet_aura,veFlip: pallet_randomness_collective_flip,
		Grandpa: pallet_grandpa,amp,

		Sudo: pallet_sudo,alances,
		TransactionPayment: pallet_transaction_payment,

		// Pallets for EVM ator_set,
		EVM: pallet_evm,et_authorship,
		Ethereum: pallet_ethereum,
		BaseFee: pallet_base_fee,
		Grandpa: pallet_grandpa,
		ImOnline: pallet_im_online,
		Sudo: pallet_sudo,
		// Governance Pallets
		Treasury: pallet_treasury,
		// Pallets for EVM 
		// Governancevm,
		Democracy: pallet_democracy,
		Scheduler: pallet_scheduler,
		Council: pallet_collective::<Instance1>,
		TechnicalCommittee: pallet_collective::<Instance2>,
		Preimage: pallet_preimage,
	}// Governance Pallets
);Treasury: pallet_treasury,

pub struct TransactionConverter;
		Democracy: pallet_democracy,
impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
		UncheckedExtrinsic::new_unsigned(tive::<Instance2>,
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		)
	}
}
pub struct TransactionConverter;
impl fp_rpc::ConvertTransaction<opaque::UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(saction<UncheckedExtrinsic> for TransactionConverter {
		&self,ert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
		transaction: pallet_ethereum::Transaction,
	) -> opaque::UncheckedExtrinsic {>::transact { transaction }.into(),
		let extrinsic = UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		);
		let encoded = extrinsic.encode();
		opaque::UncheckedExtrinsic::decode(&mut &encoded[..])sic> for TransactionConverter {
			.expect("Encoded extrinsic is always valid")
	}&self,
}	transaction: pallet_ethereum::Transaction,
	) -> opaque::UncheckedExtrinsic {
/// The address format for describing accounts.ned(
// pub type Address = sp_runtime::MultiAddress<AccountId, ()>;into(),
pub type Address = AccountId;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.id")
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,nts.
	frame_system::CheckSpecVersion<Runtime>,dress<AccountId, ()>;
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,this runtime.
	frame_system::CheckEra<Runtime>,<BlockNumber, BlakeTwo256>;
	frame_system::CheckNonce<Runtime>,runtime.
	frame_system::CheckWeight<Runtime>,er, UncheckedExtrinsic>;
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);b type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
// impl pallet_offences::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;>,
// 	type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
// 	type OnOffenceHandler = Staking;
// }me_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

// impl pallet_offences::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
/// Unchecked extrinsic type as expected by this runtime.::IdentificationTuple<Self>;
pub type UncheckedExtrinsic =taking;
fp_self_contained::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;

/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,hecked extrinsic type as expected by this runtime.
	frame_system::ChainContext<Runtime>,
	Runtime,ontained::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
	AllPalletsWithSystem,
>;/ The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
#[cfg(feature = "runtime-benchmarks")] various modules.
#[macro_use]cutive = frame_executive::Executive<
extern crate frame_benchmarking;
	Block,
#[cfg(feature = "runtime-benchmarks")]
mod benches {
	define_benchmarks!(m,
		[frame_benchmarking, BaselineBench::<Runtime>]
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]hmarks")]
		[pallet_timestamp, Timestamp]
		[pallet_im_online, ImOnline]g;
		[pallet_treasury, Treasury]
		[pallet_democracy, Democracy]arks")]
		[pallet_collective, Council]
		[pallet_scheduler, Scheduler]
		[pallet_preimage, Preimage]neBench::<Runtime>]
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
	);pallet_timestamp, Timestamp]
}	[pallet_im_online, ImOnline]
		[pallet_treasury, Treasury]
impl fp_self_contained::SelfContainedCall for RuntimeCall {
	type SignedInfo = H160;uncil]
		[pallet_scheduler, Scheduler]
	fn is_self_contained(&self) -> bool {
		match self {
			RuntimeCall::Ethereum(call) => call.is_self_contained(),
			_ => false,
		}
	}
impl fp_self_contained::SelfContainedCall for RuntimeCall {
	fn check_self_contained(&self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) => call.check_self_contained(),
			_ => None,{
		}RuntimeCall::Ethereum(call) => call.is_self_contained(),
	}	_ => false,
		}
	fn validate_self_contained(
		&self,
		info: &Self::SignedInfo,self) -> Option<Result<Self::SignedInfo, TransactionValidityError>> {
		dispatch_info: &DispatchInfoOf<RuntimeCall>,
		len: usize,l::Ethereum(call) => call.check_self_contained(),
	) -> Option<TransactionValidity> {
		match self {
			RuntimeCall::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
			_ => None,
		} validate_self_contained(
	}&self,
		info: &Self::SignedInfo,
	fn apply_self_contained(hInfoOf<RuntimeCall>,
		self,usize,
		info: Self::SignedInfo,alidity> {
	) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
		match self {::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
			call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) => Some(call.dispatch(
				RuntimeOrigin::from(pallet_ethereum::RawOrigin::EthereumTransaction(info)),
			)),
			_ => None,
		} apply_self_contained(
	}self,
}	info: Self::SignedInfo,
	) -> Option<sp_runtime::DispatchResultWithInfo<PostDispatchInfoOf<Self>>> {
impl_runtime_apis! {
			call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) => Some(call.dispatch(
	impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {reumTransaction(info)),
			)),
		fn gas_limit_multiplier_support() {}
		}
		fn chain_id() -> u64 {
			<Runtime as pallet_evm::Config>::ChainId::get()
		}
impl_runtime_apis! {
		fn account_basic(address: H160) -> EVMAccount {
			let (account, _) = EVM::account_basic(&address);ime {
			// EVM::account_basic(&address)
			accountimit_multiplier_support() {}
		}
		fn chain_id() -> u64 {
		fn gas_price() -> U256 {:Config>::ChainId::get()
			let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();
			gas_price
		}n account_basic(address: H160) -> EVMAccount {
			let (account, _) = EVM::account_basic(&address);
		fn account_code_at(address: H160) -> Vec<u8> {
			EVM::account_codes(address)
		}

		fn author() -> H160 {6 {
			<pallet_evm::Pallet<Runtime>>::find_author():Config>::FeeCalculator::min_gas_price();
		}gas_price
		}
		fn storage_at(address: H160, index: U256) -> H256 {
			let mut tmp = [0u8; 32];s: H160) -> Vec<u8> {
			index.to_big_endian(&mut tmp);
			EVM::account_storages(address, H256::from_slice(&tmp[..]))
		}
		fn author() -> H160 {
		fn call(_evm::Pallet<Runtime>>::find_author()
			from: H160,
			to: H160,
			data: Vec<u8>,ddress: H160, index: U256) -> H256 {
			value: U256,= [0u8; 32];
			gas_limit: U256,ian(&mut tmp);
			max_fee_per_gas: Option<U256>, H256::from_slice(&tmp[..]))
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
				let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)as: Option<U256>,
			} else {rity_fee_per_gas: Option<U256>,
				None: Option<U256>,
			};timate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
			let is_transactional = false;fo, sp_runtime::DispatchError> {
			<Runtime as pallet_evm::Config>::Runner::call(
				from,ut config = <Runtime as pallet_evm::Config>::config().clone();
				to,fig.estimate = true;
				data,config)
				value,{
				gas_limit.unique_saturated_into(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,transactional = false;
				access_list.unwrap_or_default(),Runner::call(
				is_transactional,
				estimate,
				config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
			).map_err(|err| err.error.into())
		}	gas_limit.unique_saturated_into(),
				max_fee_per_gas,
		fn elasticity() -> Option<Permill> {
			Some(BaseFee::elasticity())
		}	access_list.unwrap_or_default(),
				is_transactional,
		fn create(,
			from: H160,ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
			data: Vec<u8>,| err.error.into())
			value: U256,
			gas_limit: U256,
			gas_price: Option<U256>,<Permill> {
			max_priority_fee: Option<U256>,
			nonce: Option<U256>,
			estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>
		) -> Result<fp_evm::CreateInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as pallet_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)tion<U256>,
			} else {rity_fee: Option<U256>,
				None: Option<U256>,
			};timate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>
			let is_transactional = false;, sp_runtime::DispatchError> {
			<Runtime as pallet_evm::Config>::Runner::create(
				from,ut config = <Runtime as pallet_evm::Config>::config().clone();
				data,g.estimate = true;
				value,onfig)
				gas_limit.low_u64(),
				gas_price,
				max_priority_fee,
				nonce,
				access_list.unwrap_or_default(),
				is_transactional,_evm::Config>::Runner::create(
				estimate,
				config.as_ref().unwrap_or(<Runtime as pallet_evm::Config>::config()),
			).map_err(|err| err.error.into())
		}	gas_limit.low_u64(),
				gas_price,
		fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
			Ethereum::current_transaction_statuses()
		}	access_list.unwrap_or_default(),
				is_transactional,
		fn current_block() -> Option<pallet_ethereum::Block> {
			Ethereum::current_block()r(<Runtime as pallet_evm::Config>::config()),
		}).map_err(|err| err.error.into())
		}
		fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
			Ethereum::current_receipts()es() -> Option<Vec<TransactionStatus>> {
		}Ethereum::current_transaction_statuses()
		}
		fn current_all() -> (
			Option<pallet_ethereum::Block>,let_ethereum::Block> {
			Option<Vec<pallet_ethereum::Receipt>>,
			Option<Vec<TransactionStatus>>
		) {
			( current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
				Ethereum::current_block(),)
				Ethereum::current_receipts(),
				Ethereum::current_transaction_statuses()
			) current_all() -> (
		}Option<pallet_ethereum::Block>,
			Option<Vec<pallet_ethereum::Receipt>>,
		fn extrinsic_filter(ionStatus>>
			xts: Vec<<Block as BlockT>::Extrinsic>,
		) -> Vec<EthereumTransaction> {
			xts.into_iter().filter_map(|xt| match xt.0.function {
				RuntimeCall::Ethereum(transact{transaction}) => Some(transaction),
				_ => None:current_transaction_statuses()
			}).collect::<Vec<EthereumTransaction>>()
		}
	}
		fn extrinsic_filter(
	impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
		fn convert_transaction(transaction: EthereumTransaction) -> <Block as BlockT>::Extrinsic {
			UncheckedExtrinsic::new_unsigned(atch xt.0.function {
				pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
			)_ => None
		}}).collect::<Vec<EthereumTransaction>>()
	}}
	}
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {ntimeApi<Block> for Runtime {
			VERSIONrt_transaction(transaction: EthereumTransaction) -> <Block as BlockT>::Extrinsic {
		}UncheckedExtrinsic::new_unsigned(
				pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}VERSION
	}}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}
	}fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}pl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()sic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
		}Executive::apply_extrinsic(extrinsic)
		}
		fn check_inherents(
			block: Block,ock() -> <Block as BlockT>::Header {
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}n inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
	}	data.create_extrinsics()
		}
	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,lt {
		) -> TransactionValidity {ock)
			Executive::validate_transaction(source, tx, block_hash)
		}
	}
	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}block_hash: <Block as BlockT>::Hash,
	}) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}pl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
		fn authorities() -> Vec<AuraId> {r)
			Aura::authorities().into_inner()
		}
	}
	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
	impl sp_session::SessionKeys<Block> for Runtime {ation {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {ation())
			opaque::SessionKeys::generate(seed)
		}
		fn authorities() -> Vec<AuraId> {
		fn decode_session_keys(to_inner()
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}pl sp_session::SessionKeys<Block> for Runtime {
	}fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
	impl fg_primitives::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> GrandpaAuthorityList {
			Grandpa::grandpa_authorities()
		}encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
		fn current_set_id() -> fg_primitives::SetId {keys(&encoded)
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic({
			_equivocation_proof: fg_primitives::EquivocationProof<
				<Block as BlockT>::Hash,ies()
				NumberFor<Block>,
			>,
			_key_owner_proof: fg_primitives::OpaqueKeyOwnershipProof,
		) -> Option<()> {_set_id()
			None
		}
		fn submit_report_equivocation_unsigned_extrinsic(
		fn generate_key_ownership_proof(ves::EquivocationProof<
			_set_id: fg_primitives::SetId,
			_authority_id: GrandpaId,
		) -> Option<fg_primitives::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}fn generate_key_ownership_proof(
			_set_id: fg_primitives::SetId,
	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		fn account_nonce(account: AccountId) -> Index {oof> {
			System::account_nonce(account)ementation possible since we've
		}// defined our key owner proof type as a bottom type (i.e. a type
	}	// with no values).
			None
	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index> for Runtime {
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}len: u32,
	}) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtimee_details(
	{	uxt: <Block as BlockT>::Extrinsic,
		fn query_call_info(
			call: RuntimeCall,tion_payment::FeeDetails<Balance> {
			len: u32,onPayment::query_fee_details(uxt, len)
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(ment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}len: u32,
	}) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,ls<Balance> {
		) {ansactionPayment::query_call_fee_details(call, len)
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;
	impl frame_benchmarking::Benchmark<Block> for Runtime {
			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);arkList>,
			Vec<frame_support::traits::StorageInfo>,
			let storage_info = AllPalletsWithSystem::storage_info();
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
			(list, storage_info)raits::StorageInfoTrait;
		}use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch, TrackedStorageKey};
			let storage_info = AllPalletsWithSystem::storage_info();
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;
			use pallet_evm::Pallet as PalletEvmBench;

			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}kConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_support::traits::WhitelistedStorageKeys;enchmarkBatch, TrackedStorageKey};
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();
			use frame_system_benchmarking::Pallet as SystemBench;
			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);Bench;
			add_benchmarks!(params, batches, pallet_evm, PalletEvmBench::<Runtime>);
			impl frame_system_benchmarking::Config for Runtime {}
			Ok(batches)ne::Config for Runtime {}
		}
	}	use frame_support::traits::WhitelistedStorageKeys;
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();
	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade() -> (Weight, Weight) {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			let weight = Executive::try_runtime_upgrade().unwrap();
			(weight, BlockWeights::get().max_block)
		}
	#[cfg(feature = "try-runtime")]
		fn execute_block(time::TryRuntime<Block> for Runtime {
			block: Block,upgrade() -> (Weight, Weight) {
			state_root_check: bool,wrap: we don't want to propagate the error backwards, and want to
			select: frame_try_runtime::TryStateSelectre/post migration checks fail, we shall stop
		) -> Weight {e and right now.
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.t().max_block)
			Executive::try_execute_block(block, state_root_check, select).expect("execute-block failed")
		}
	}fn execute_block(
}		block: Block,
			state_root_check: bool,
#[cfg(test)]rame_try_runtime::TryStateSelect
mod tests {ht {
	use super::*;tentional unwrap: we don't want to propagate the error backwards, and want to
	use frame_support::traits::WhitelistedStorageKeys;
	use sp_core::hexdisplay::HexDisplay;, state_root_check, select).expect("execute-block failed")
	use std::collections::HashSet;
	}
	#[test]
	fn check_whitelist() {
		let whitelist: HashSet<String> = AllPalletsWithSystem::whitelisted_storage_keys()
			.iter(){
			.map(|e| HexDisplay::from(&e.key).to_string())
			.collect();port::traits::WhitelistedStorageKeys;
	use sp_core::hexdisplay::HexDisplay;
		// Block Numberions::HashSet;
		assert!(
			whitelist.contains("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac")
		);check_whitelist() {
		// Total IssuanceshSet<String> = AllPalletsWithSystem::whitelisted_storage_keys()
		assert!(
			whitelist.contains("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80")
		);collect();
		// Execution Phase
		assert!( Number
			whitelist.contains("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a")
		);hitelist.contains("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac")
		// Event Count
		assert!( Issuance
			whitelist.contains("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850")
		);hitelist.contains("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80")
		// System Events
		assert!(tion Phase
			whitelist.contains("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7")
		);hitelist.contains("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a")
	});
}	// Event Count
		assert!(
			whitelist.contains("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850")
		);
		// System Events
		assert!(
			whitelist.contains("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7")
		);
	}
}
