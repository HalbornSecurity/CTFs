#![cfg_attr(not(feature = "std"), no_std)]

//! Handle the ability to notify other pallets that they should stop all
//! operations, or resume them

#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::WeightInfo;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        type PauseOrigin: EnsureOrigin<Self::Origin>;
        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Toggle the shutdown state if authorized to do so.
        #[pallet::weight(T::WeightInfo::pause_base())]
        pub fn toggle(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            T::PauseOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)?;

            <Paused<T>>::put(Self::paused());
            Self::deposit_event(Event::StatusChanged(Self::paused()));

            Ok(().into())
        }

        #[pallet::weight(T::WeightInfo::pause_base())]
        pub fn pause(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            T::PauseOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)?;
            
            <Paused<T>>::put(true);
            
            Self::deposit_event(Event::StatusChanged(true));
    
            Ok(().into())
        }
    
        /// Unpause
        #[pallet::weight(T::WeightInfo::pause_base())]
        pub fn unpause(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
            T::PauseOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)?;
    
            <Paused<T>>::put(Self::paused());
            Self::deposit_event(Event::StatusChanged(false));
    
            Ok(().into())
        }

        
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    #[pallet::metadata()]
    pub enum Event<T: Config> {
        /// Shutdown state was toggled, to either on or off.
        StatusChanged(bool),
    }

    #[pallet::storage]
    #[pallet::getter(fn paused)]
    pub type Paused<T: Config> = StorageValue<_, bool, ValueQuery>;
}
