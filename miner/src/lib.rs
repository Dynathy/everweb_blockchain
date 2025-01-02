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

    /// Tracks registered miners and their deposits.
    #[pallet::storage]
    #[pallet::getter(fn miners)]
    pub type Miners<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, BalanceOf<T>, OptionQuery>;

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
    /// Tracks submitted hashes and associated metadata.
	#[pallet::storage]
	#[pallet::getter(fn submissions)]
	pub type Submissions<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::Hash,
		(T::AccountId, BoundedVec<u8, T::MaxUrlLength>),
		OptionQuery
	>;
    /// Events emitted by the pallet.
    #[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		MinerRegistered { miner: T::AccountId, deposit: BalanceOf<T> },
		SubmissionAccepted { miner: T::AccountId, url: Vec<u8>, hash: T::Hash },

		//Embedded Whitelist
		WhitelistUpdated { url: Vec<u8>, added: bool }, // Added for whitelist changes
	}

    /// Errors that can occur in the pallet.
    #[pallet::error]
    pub enum Error<T> {
        MinerAlreadyRegistered,
        MinerNotRegistered,
        NotWhitelisted,
        InsufficientFunds,
		UrlTooLong, // Too Long error

		//Embedded Whitelist 
		UrlAlreadyWhitelisted, // New error
    	UrlNotWhitelisted, // New error for removal
    }

	impl<T: Config> Pallet<T> {
		/// Returns the account ID for the pallet
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}

    /// Pallet calls.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new miner with a deposit.
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn register_miner(origin: OriginFor<T>, deposit: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(!Miners::<T>::contains_key(&who), Error::<T>::MinerAlreadyRegistered);

            T::Currency::reserve(&who, deposit)?;
            Miners::<T>::insert(&who, deposit);
			log::info!("About to deposit event for miner registration");
            Self::deposit_event(Event::MinerRegistered { miner: who.clone(), deposit });
			log::info!("Event deposited");
            Ok(())
        }

        /// Submit a hash for validation.
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn submit_hash(origin: OriginFor<T>, url: Vec<u8>, hash: T::Hash) -> DispatchResult {
			let miner = ensure_signed(origin)?;

			// Convert `url` to `BoundedVec`
			let bounded_url: BoundedVec<u8, T::MaxUrlLength> =
				url.clone().try_into().map_err(|_| Error::<T>::UrlTooLong)?;
	
			// Ensure the URL is whitelisted
			ensure!(Whitelist::<T>::contains_key(&bounded_url), Error::<T>::NotWhitelisted);
	
			// Insert the submission
			Submissions::<T>::insert(hash, (&miner, bounded_url));
	
			Self::deposit_event(Event::SubmissionAccepted { miner, url, hash });
	
			Ok(())
        }

		///Embedded Whitelist placeholders
		 /// Add a URL to the whitelist.
		 #[pallet::call_index(2)]
		 #[pallet::weight(10_000)]
		 pub fn add_to_whitelist(origin: OriginFor<T>, url: Vec<u8>) -> DispatchResult {
			 ensure_root(origin)?; // Only root can modify the whitelist
	 
			 // Convert URL to a bounded vector
			 let bounded_url: BoundedVec<u8, T::MaxUrlLength> =
				 url.clone().try_into().map_err(|_| Error::<T>::UrlTooLong)?;
	 
			 // Ensure the URL is not already whitelisted
			 ensure!(!Whitelist::<T>::contains_key(&bounded_url), Error::<T>::UrlAlreadyWhitelisted);
	 
			 // Add to the whitelist
			 Whitelist::<T>::insert(&bounded_url, ());
			 Self::deposit_event(Event::WhitelistUpdated { url, added: true });
			 Ok(())
		 }
	 
		 /// Remove a URL from the whitelist.
		 #[pallet::call_index(3)]
		 #[pallet::weight(10_000)]
		 pub fn remove_from_whitelist(origin: OriginFor<T>, url: Vec<u8>) -> DispatchResult {
			 ensure_root(origin)?; // Only root can modify the whitelist
	 
			 // Convert URL to a bounded vector
			 let bounded_url: BoundedVec<u8, T::MaxUrlLength> =
				 url.clone().try_into().map_err(|_| Error::<T>::UrlTooLong)?;
	 
			 // Ensure the URL is already whitelisted
			 ensure!(Whitelist::<T>::contains_key(&bounded_url), Error::<T>::UrlNotWhitelisted);
	 
			 // Remove from the whitelist
			 Whitelist::<T>::remove(&bounded_url);
			 Self::deposit_event(Event::WhitelistUpdated { url, added: false });
			 Ok(())
		 }
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;