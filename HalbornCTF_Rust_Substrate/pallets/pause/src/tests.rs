#![cfg(test)]

use super::*;
use crate::{self as pallet_pause};
use frame_support::{assert_noop, assert_ok, ord_parameter_types, parameter_types};
use frame_system::{EnsureSignedBy, RawOrigin};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    DispatchError::BadOrigin,
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
    pub enum Test where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        TestModule: pallet_pause::{Module, Call, Storage, Event<T>},
    }
);

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}
impl frame_system::Config for Test {
    type Origin = Origin;
    type Call = Call;
    type BlockWeights = ();
    type BlockLength = ();
    type SS58Prefix = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = ();
    type BlockHashCount = BlockHashCount;
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = ();
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = ();
    type SystemWeightInfo = ();
}

ord_parameter_types! {
    pub const Admin: u64 = 1;
}
impl Config for Test {
    type Event = ();
    type PauseOrigin = EnsureSignedBy<Admin, u64>;
    type WeightInfo = ();
}

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    frame_system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}

#[test]
fn root_pause() {
    new_test_ext().execute_with(|| {
        assert_ok!(TestModule::pause(RawOrigin::Root.into()));
    })
}

#[test]
fn pause_origin_unpause() {
    new_test_ext().execute_with(|| {
        assert_ok!(TestModule::unpause(Origin::signed(Admin::get())));
    })
}

#[test]
fn bad_origin_fails() {
    new_test_ext().execute_with(|| {
        assert_noop!(TestModule::pause(Origin::signed(0)), BadOrigin);
    })
}
