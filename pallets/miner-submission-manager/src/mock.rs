#![cfg(test)]

use crate as pallet_miner_submission_manager;
use frame_support::{parameter_types,
    traits::ConstU128, 
    traits::ConstU64,
    PalletId,};
use frame_support::assert_ok;
use pallet_verifier;
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
        Whitelist: pallet_whitelist, // Add the Whitelist pallet
        MinerSubmissionManager: pallet_miner_submission_manager,
        Verifier: pallet_verifier,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaxLocks: u32 = 50;
    pub const MaxUrlLength: u32 = 256;
    pub const SubmissionFee: u64 = 10;
    pub const MinerPalletId: PalletId = PalletId(*b"py/miner");
    pub const VerifierPalletId: PalletId = PalletId(*b"py/valid");
    pub const ExistentialDeposit: u128 = 1;
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

// Pallet Verifier Config
impl pallet_verifier::Config for Test {
    type Currency = Balances;
    type SubmissionFee = SubmissionFee;
    type RuntimeEvent = RuntimeEvent;
    type PalletId = VerifierPalletId;
    type MaxUrlLength = MaxUrlLength;
}

impl pallet_whitelist::Config for Test {
    type RuntimeEvent = RuntimeEvent;
    type WhitelistOrigin = frame_system::EnsureRoot<AccountId32>;
    type MaxUrlLength = MaxUrlLength;
}

impl pallet_miner_submission_manager::Config for Test {
    type Currency = Balances;
    type SubmissionFee = SubmissionFee;
    type RuntimeEvent = RuntimeEvent;
    type PalletId = MinerPalletId;
    type MaxUrlLength = MaxUrlLength;
    // Add WeightInfo if applicable
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    // Build the storage using frame_system's GenesisConfig
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    // Assimilate pallet_balances into the storage
    pallet_balances::GenesisConfig::<Test> {
        balances : vec![
            (AccountId32::new([1; 32]), 1_000),
            (AccountId32::new([2; 32]), 1_000),
            (AccountId32::new([3; 32]), 1_000),
            (AccountId32::new([4; 32]), 1_000),
            (AccountId32::new([5; 32]), 1_000),
            (AccountId32::new([6; 32]), 1_000),
            (AccountId32::new([7; 32]), 1_000),
            (AccountId32::new([8; 32]), 1_000),
            (AccountId32::new([9; 32]), 1_000),
            (AccountId32::new([10; 32]), 1_000),
            (AccountId32::new([11; 32]), 1_000),
            ],
    }
    .assimilate_storage(&mut storage)
    .unwrap();

    // Convert the storage into TestExternalities
    let mut ext = sp_io::TestExternalities::new(storage);
    ext.execute_with(|| {
        System::set_block_number(1);

        // Register verifiers
        assert_ok!(pallet_verifier::Pallet::<Test>::register_verifier(
            RuntimeOrigin::signed(AccountId32::new([1; 32])),
            100
        ));
        assert_ok!(pallet_verifier::Pallet::<Test>::register_verifier(
            RuntimeOrigin::signed(AccountId32::new([2; 32])),
            100
        ));
        assert_ok!(pallet_verifier::Pallet::<Test>::register_verifier(
            RuntimeOrigin::signed(AccountId32::new([3; 32])),
            100
        ));
        assert_ok!(pallet_verifier::Pallet::<Test>::register_verifier(
            RuntimeOrigin::signed(AccountId32::new([4; 32])),
            100
        ));
        assert_ok!(pallet_verifier::Pallet::<Test>::register_verifier(
            RuntimeOrigin::signed(AccountId32::new([5; 32])),
            100
        ));
        // Register verifiers
        assert_ok!(pallet_verifier::Pallet::<Test>::register_verifier(
            RuntimeOrigin::signed(AccountId32::new([6; 32])),
            100
        ));
        assert_ok!(pallet_verifier::Pallet::<Test>::register_verifier(
            RuntimeOrigin::signed(AccountId32::new([7; 32])),
            100
        ));
        assert_ok!(pallet_verifier::Pallet::<Test>::register_verifier(
            RuntimeOrigin::signed(AccountId32::new([8; 32])),
            100
        ));
        assert_ok!(pallet_verifier::Pallet::<Test>::register_verifier(
            RuntimeOrigin::signed(AccountId32::new([9; 32])),
            100
        ));
        assert_ok!(pallet_verifier::Pallet::<Test>::register_verifier(
            RuntimeOrigin::signed(AccountId32::new([10; 32])),
            100
        ));
        assert_ok!(pallet_verifier::Pallet::<Test>::register_verifier(
            RuntimeOrigin::signed(AccountId32::new([11; 32])),
            100
        ));
    });

    ext
}