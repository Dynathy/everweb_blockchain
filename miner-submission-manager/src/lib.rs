#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
        dispatch::DispatchResult,
        pallet_prelude::*,
		BoundedVec,
		PalletId,
        traits::{Currency, ExistenceRequirement, ReservableCurrency},
    };
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;
	use sp_runtime::traits::AccountIdConversion;

    type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);


	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Currency: ReservableCurrency<Self::AccountId>;
		type SubmissionFee: Get<BalanceOf<Self>>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		#[pallet::constant]
    	type PalletId: Get<PalletId>;
		#[pallet::constant]
    	type MaxUrlLength: Get<u32>; // Maximum length for URLs
	}

    /// Tracks submissions for validation.
    #[pallet::storage]
    #[pallet::getter(fn submissions)]
    pub type Submissions<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash,
		(T::AccountId, BoundedVec<u8, T::MaxUrlLength>),
		OptionQuery,
	>;

    /// Whitelist for valid URLs.
    #[pallet::storage]
	#[pallet::getter(fn whitelist)]
	pub type Whitelist<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxUrlLength>,
		(),
		OptionQuery
	>;

    /// Events emitted by the pallet.
    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		///New Submission Added.
		SubmissionReceived { miner: T::AccountId, hash: T::Hash, url: Vec<u8>},
		///URL added to whitelist.
		UrlWhitelisted { url: Vec<u8> },
		///URL remove from whitelist.
		UrlRemovedFromWhitelist { url: Vec<u8> },
	}

    /// Errors that can occur in the pallet.
    #[pallet::error]
    pub enum Error<T> {
        ///Submission already exists.
		DuplicateSubmission,
		///URL not whitelisted.
		NotWhitelisted,
		///URL exceeds maximum length.
		UrlTooLong
    }

	/// Pallet calls.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Miner submits a hash for validation.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn submit_hash(
            origin: OriginFor<T>,
            url: Vec<u8>,
            hash: T::Hash,
        ) -> DispatchResult {
            let miner = ensure_signed(origin)?;

            // Convert `url` to `BoundedVec`
            let bounded_url: BoundedVec<u8, T::MaxUrlLength> =
                url.clone().try_into().map_err(|_| Error::<T>::UrlTooLong)?;

            // Ensure the URL is whitelisted
            ensure!(Whitelist::<T>::contains_key(&bounded_url), Error::<T>::NotWhitelisted);

            // Ensure submission is unique
            ensure!(
                !Submissions::<T>::contains_key(&hash),
                Error::<T>::DuplicateSubmission
            );

            // Insert submission
            Submissions::<T>::insert(hash, (miner.clone(), bounded_url));

            // Emit an event
            Self::deposit_event(Event::SubmissionReceived { miner, hash, url });

            Ok(())
        }
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;