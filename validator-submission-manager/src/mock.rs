#![cfg(test)]

use crate as pallet_validator_submission_manager;
use pallet_treasury_manager::Config as TreasuryConfig;
use frame_support::{parameter_types, traits::ConstU128, traits::ConstU64};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
    AccountId32,
};
use sp_runtime::traits::AccountIdConversion;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system,
        Balances: pallet_balances,
        Treasury: pallet_treasury, // Add this line
        ValidatorSubmissionManager: pallet_validator_submission_manager,
        TreasuryManager: pallet_treasury_manager, // Added this line
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaxLocks: u32 = 50;
    pub const MaxValidatorSubmissions: u32 = 10;
    pub const ValidationTimeOut: u64 = 10;
    pub const SubmissionFee: u64 = 10;
    pub const ExistentialDeposit: u128 = 1;
    pub const ValidatorSubmissionManagerPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/vlmgr");
    pub const TreasuryPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/trsry");
    pub const TreasuryManagerPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/trman");
    pub const DevPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/devid");
    pub const DefaultDevAccount: AccountId32 = AccountId32::new([0u8; 32]); // Example account
    pub const FeeSplitTreasury: u8 = 90; // Treasury receives 70%
    pub const MinerRewardPercentage: u8 = 90; // Miner gets 50%
    pub const ValidatorRewardPercentage: u8 = 10; // Validators share 50%
    pub const TotalReward: u128 = 1000; // Total reward distributed
}

// Frame System Config
impl frame_system::Config for Test {
    type BaseCallFilter = frame_support::traits::Everything;
    type BlockWeights = ();
    type BlockLength = ();
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type AccountId = AccountId32;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type RuntimeEvent = RuntimeEvent;
    type Block = Block;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<u128>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type RuntimeTask = (); // Default to `()`, adjust as necessary
    type BlockHashCount = ConstU64<250>;
    type DbWeight = ();
    type MaxConsumers = frame_support::traits::ConstU32<16>;
    type SingleBlockMigrations = ();
    type MultiBlockMigrator = ();
    type PreInherents = ();
    type PostInherents = ();
    type PostTransactions = ();
}

impl pallet_balances::Config for Test {
    type MaxLocks = ();
    type Balance = u128;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ConstU128<1>;
    type AccountStore = System;
    type WeightInfo = ();
    type RuntimeHoldReason = (); // Default to `()`, adjust as necessary
    type RuntimeFreezeReason = (); // Default to `()`, adjust as necessary
    type ReserveIdentifier = [u8; 8];
    type FreezeIdentifier = [u8; 8];
    type MaxReserves = frame_support::traits::ConstU32<1>;
    type MaxFreezes = frame_support::traits::ConstU32<1>;
}

impl pallet_treasury_manager::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type TreasuryCurrency = Balances;
    type TreasuryPalletId = TreasuryPalletId;
    type DevPalletId = DevPalletId;
    type DefaultDevAccount = DefaultDevAccount;
    type FeeSplitTreasury = FeeSplitTreasury;
    type MinerRewardPercentage = MinerRewardPercentage;
    type ValidatorRewardPercentage = ValidatorRewardPercentage;
    type TotalReward = TotalReward;
    type RootOrigin = frame_system::EnsureRoot<AccountId32>; // Added RootOrigin type
}

impl pallet_treasury::Config for Test {
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type PalletId = TreasuryPalletId;
}

impl pallet_validator_submission_manager::Config for Test {
    type TreasuryManager = Test;
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type MaxValidatorSubmissions = MaxValidatorSubmissions;
    type ValidationTimeout = ConstU64<5>; // Timeout in blocks
    type TotalReward = TotalReward;
    //type WeightInfo = ();
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    // Build the storage using frame_system's GenesisConfig
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    // Retrieve the treasury account
    let treasury_account: AccountId32 = TreasuryPalletId::get().into_account_truncating();
    let dev_account: AccountId32 = DevPalletId::get().into_account_truncating();

     // Assimilate pallet_balances into the storage
     pallet_balances::GenesisConfig::<Test> {
        balances: vec![
            (TreasuryPalletId::get().into_account_truncating(), 1_000_000),
            (DevPalletId::get().into_account_truncating(), 100_000),
        ],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        crate::ValidatorSubmissions::<Test>::remove_all(None);
        crate::ValidationDeadline::<Test>::remove_all(None);
        crate::ProcessedSubmissions::<Test>::remove_all(None);
        crate::MinerForHash::<Test>::remove_all(None);
        System::set_block_number(1);
        log::info!("Treasury account initialized: {:?}", treasury_account);
        log::info!(
            "Treasury balance: {:?}",
            pallet_balances::Pallet::<Test>::free_balance(&treasury_account)
        );
        log::info!("Dev account initialized: {:?}", dev_account);
        log::info!(
            "Dev balance: {:?}",
            pallet_balances::Pallet::<Test>::free_balance(&dev_account)
        );
    });
    ext
}