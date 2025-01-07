#![cfg(test)]

use crate as pallet_miner;
use frame_support::{parameter_types, traits::ConstU128, traits::ConstU64};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    BuildStorage,
    AccountId32,
};


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
        SubmissionManager: pallet_miner_submission_manager, // Ensure SubmissionManager is above Miner
        Miner: pallet_miner,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaxLocks: u32 = 50;
    pub const MaxUrlLength: u32 = 256;
    pub const SubmissionFee: u64 = 10;
    pub const MinerPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/miner");
    pub const ManagerPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/mmngr");
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


impl pallet_miner::Config for Test {
    type Currency = Balances;
    type SubmissionFee = SubmissionFee;
    type RuntimeEvent = RuntimeEvent;
    type PalletId = MinerPalletId;
    type MaxUrlLength = MaxUrlLength;
    //type WeightInfo = ();
}

impl pallet_miner_submission_manager::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type PalletId = ManagerPalletId;
    type MaxUrlLength = MaxUrlLength;
    type Currency = Balances;
    type SubmissionFee = SubmissionFee;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    // Build the storage using frame_system's GenesisConfig
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    // Assimilate pallet_balances into the storage
    pallet_balances::GenesisConfig::<Test> {
        balances: vec![(AccountId32::new([1; 32]), 1_000)],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    // Convert the storage into TestExternalities
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| System::set_block_number(1)); // Ensure a block number is set
    ext
}