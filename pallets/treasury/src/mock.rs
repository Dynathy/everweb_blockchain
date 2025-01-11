#![cfg(test)]

use crate as pallet_treasury;
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
        Treasury: pallet_treasury,
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const TreasuryPalletId: frame_support::PalletId = frame_support::PalletId(*b"py/trsry");
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

// Balances Pallet Config
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

// Treasury Pallet Config
impl pallet_treasury::Config for Test {
    type Currency = Balances;
    type RuntimeEvent = RuntimeEvent;
    type PalletId = TreasuryPalletId;
}

pub(crate) fn new_test_ext() -> sp_io::TestExternalities {
    let mut storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();

    
    
    let mut ext = sp_io::TestExternalities::from(storage);
    ext.execute_with(|| System::set_block_number(1));
    ext
}
