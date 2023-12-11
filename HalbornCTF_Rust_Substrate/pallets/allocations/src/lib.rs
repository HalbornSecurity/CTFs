#![cfg_attr(not(feature = "std"), no_std)]

mod benchmarking;
#[cfg(test)]
mod tests;

use sp_std::prelude::*;

use frame_support::{
    ensure,
    traits::{ChangeMembers, Currency, Get, InitializeMembers},
};
use frame_system::ensure_signed;
use malborn_support::WithAccountId;

use sp_runtime::{
    traits::{CheckedAdd, Saturating},
    DispatchResult, Perbill,
};

pub mod weights;
pub use weights::WeightInfo;

pub use pallet::*;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_pause::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type Currency: Currency<Self::AccountId>;

        #[pallet::constant]
        type ProtocolFee: Get<Perbill>;
        type ProtocolFeeReceiver: WithAccountId<Self::AccountId>;

        #[pallet::constant]
        type MaximumCoinsEverAllocated: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type ExistentialDeposit: Get<BalanceOf<Self>>;

        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        /// Allocate funds from the oracle to a specified account, considering protocol fees and existential deposits.
        /// 
        /// # Arguments
        /// * `origin`: The origin of the call, ensuring it is from an authorized oracle.
        /// * `to`: The account ID to which the funds will be allocated.
        /// * `amount`: The amount of funds to be allocated.
        /// * `_proof`: A vector of bytes representing a proof (unused in this implementation).
        /// 
        /// # Returns
        /// Returns a `DispatchResultWithPostInfo` indicating the success or failure of the operation, along with post-execution information.
        #[pallet::weight(
			<T as pallet::Config>::WeightInfo::allocate(_proof.len() as u32)
		)]
        pub fn allocate_coins(
            origin: OriginFor<T>,
            to: T::AccountId,
            amount: BalanceOf<T>,
            _proof: Vec<u8>,
        ) -> DispatchResultWithPostInfo {
            Self::validate_oracle(origin)?;

            let to_consume_coins = Self::allocated_coins() + amount;

            let amount_for_protocol = T::ProtocolFee::get() * amount;
            let amount_for_grantee = amount.saturating_sub(amount_for_protocol);


            <CoinsAllocations<T>>::put(to_consume_coins);

            ensure!(
                to_consume_coins < T::MaximumCoinsEverAllocated::get(),
                Error::<T>::TooManyCoinsToAllocate
            );

            T::Currency::resolve_creating(
                &T::ProtocolFeeReceiver::account_id(),
                T::Currency::issue(amount_for_protocol),
            );
            T::Currency::resolve_creating(&to, T::Currency::issue(amount_for_grantee));

            Ok(().into())
        }

    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "Balance")]
    pub enum Event<T: Config> {
        NewCoinAllocation(T::AccountId, BalanceOf<T>, BalanceOf<T>, Vec<u8>),
    }

    #[pallet::error]
    pub enum Error<T> {
        OracleAccessDenied,
        TooManyCoinsToAllocate,
    }

    #[pallet::storage]
    #[pallet::getter(fn oracles)]
    pub type Oracles<T: Config> = StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn allocated_coins)]
    pub type CoinsAllocations<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;
}

impl<T: Config> Pallet<T> {
    pub fn is_oracle(who: T::AccountId) -> bool {
        Self::oracles().contains(&who)
    }

    fn validate_oracle(origin: T::Origin) -> DispatchResult {
        let sender = ensure_signed(origin)?;
        ensure!(Self::is_oracle(sender), Error::<T>::OracleAccessDenied);
        Ok(())
    }

 
}

impl<T: Config> InitializeMembers<T::AccountId> for Pallet<T> {
    fn initialize_members(init: &[T::AccountId]) {
        <Oracles<T>>::put(init);
    }
}
