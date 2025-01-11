#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
        traits::EnsureOrigin,
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The origin which is allowed to modify the whitelist.
        type WhitelistOrigin: EnsureOrigin<Self::RuntimeOrigin>;

        /// Maximum length of the URLs.
        #[pallet::constant]
        type MaxUrlLength: Get<u32>;

        /// The runtime event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    /// Storage for the whitelist.
    #[pallet::storage]
    #[pallet::getter(fn whitelist)]
    pub type Whitelist<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, T::MaxUrlLength>, // URL
        (),                              // Empty value (just key existence)
        OptionQuery
    >;

    /// Events emitted by the pallet.
    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// URL was added to the whitelist.
        UrlAdded { url: Vec<u8> },
        /// URL was removed from the whitelist.
        UrlRemoved { url: Vec<u8> },
    }

    /// Errors that can occur in the pallet.
    #[pallet::error]
    pub enum Error<T> {
        /// The URL is already whitelisted.
        UrlAlreadyWhitelisted,
        /// The URL is not whitelisted.
        UrlNotWhitelisted,
        /// The URL is too long.
        UrlTooLong,
    }

    impl<T: Config> Pallet<T> {
        /// Check if a URL is whitelisted.
        pub fn is_whitelisted(url: Vec<u8>) -> Result<bool, Error<T>> {
            // Convert URL to BoundedVec
            let bounded_url: BoundedVec<u8, T::MaxUrlLength> =
                url.try_into().map_err(|_| Error::<T>::UrlTooLong)?;

            // Check existence in storage
            Ok(Whitelist::<T>::contains_key(&bounded_url))
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Add a URL to the whitelist.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn add_url(origin: OriginFor<T>, url: Vec<u8>) -> DispatchResult {
            log::info!("add_url called with URL: {:?}", url);
        
            T::WhitelistOrigin::ensure_origin(origin)?;
            log::info!("Origin check passed");
        
            let bounded_url: BoundedVec<u8, T::MaxUrlLength> =
                url.clone().try_into().map_err(|_| Error::<T>::UrlTooLong)?;
            log::info!("Converted URL to BoundedVec: {:?}", bounded_url);
        
            ensure!(!Whitelist::<T>::contains_key(&bounded_url), Error::<T>::UrlAlreadyWhitelisted);
            log::info!("URL is not already whitelisted");
        
            Whitelist::<T>::insert(&bounded_url, ());
            log::info!("URL inserted into whitelist storage");
            log::info!("Whitelist state inside of add_url: {:?}", Whitelist::<T>::iter().collect::<Vec<_>>());
        
            Self::deposit_event(Event::UrlAdded { url });
            log::info!("Event deposited: UrlAdded");
        
            Ok(())
        }

        /// Remove a URL from the whitelist.
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn remove_url(origin: OriginFor<T>, url: Vec<u8>) -> DispatchResult {
            T::WhitelistOrigin::ensure_origin(origin)?;

            let bounded_url: BoundedVec<u8, T::MaxUrlLength> =
                url.clone().try_into().map_err(|_| Error::<T>::UrlTooLong)?;

            ensure!(Whitelist::<T>::contains_key(&bounded_url), Error::<T>::UrlNotWhitelisted);

            Whitelist::<T>::remove(&bounded_url);
            Self::deposit_event(Event::UrlRemoved { url });
            Ok(())
        }
    }
}
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;