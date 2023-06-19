#![cfg_attr(not(feature = "std"), no_std)]
#![feature(associated_type_defaults)]
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
use xcm::prelude::*;
use scale_info::prelude::boxed::Box;
use orml_traits::location::Reserve;
use orml_traits::arithmetic::Zero;
use xcm_executor::traits::Convert;
use frame_support::traits::Contains;
use orml_traits::xcm_transfer::Transferred;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
use xcm::VersionedMultiLocation;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_uniques::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;
		type AccountIdToMultiLocation: Convert<Self::AccountId, MultiLocation>;
		type MaxAssetsForTransfer: Get<usize>;
		type MultiLocationsFilter: Contains<MultiLocation>;
		type ReserveProvider: Reserve;
		

	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored { something: u32, who: T::AccountId },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		NFTNotFound,
		BadVersion,
		TooManyAssetsBeingSent,
		InvalidAsset,
		NotSupportedMultiLocation,
		AssetIndexNonExistent
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T>where xcm::v3::MultiLocation: From<<T as pallet_uniques::Config>::CollectionId> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::call_index(0)]
		#[pallet::weight(<T as pallet::Config>::WeightInfo::do_something())]
		pub fn transfer(origin: OriginFor<T>, 
			collection_id:T::CollectionId, 
			target_chain:Box<VersionedMultiLocation>) -> DispatchResult {
				let who = ensure_signed(origin)?;
	
			let collection_details = pallet_uniques::Collection::<T>::get(&collection_id)
            .ok_or(Error::<T>::NFTNotFound)?;
		let target_chain: MultiLocation = (*target_chain).try_into().map_err(|()| Error::<T>::BadVersion)?;
		Self::do_transfer(who, collection_id, target_chain).map(|_| ())
			// Self::deposit_event(Event::NftTransfered { something, who });
			// Return a successful DispatchResultWithPostInfo	
			// Ok(())
		}

		
	}


	impl<T: Config> Pallet<T> where xcm::v3::MultiLocation: From<<T as pallet_uniques::Config>::CollectionId>{
		fn do_transfer(
			who: T::AccountId,
			collection_id: T::CollectionId,
			target_chain: MultiLocation,
		) -> Result<Transferred<T::AccountId>, DispatchError>{

			let location: MultiLocation = collection_id.into();
			let fungibility: Fungibility = Fungibility::Fungible(1u128);
  	 	 let asset: MultiAsset = (location, fungibility).into();
			Self::do_transfer_multiassets(who, vec![asset.clone()].into(), asset, target_chain)
			
		}

		fn do_transfer_multiassets(
			who: T::AccountId,
			assets: MultiAssets,
			fee: MultiAsset,
			target_chain: MultiLocation,
		) -> Result<Transferred<T::AccountId>, DispatchError> {
			ensure!(
				assets.len() <= T::MaxAssetsForTransfer::get(),
				Error::<T>::TooManyAssetsBeingSent
			);
			ensure!(
				T::MultiLocationsFilter::contains(&target_chain),
				Error::<T>::NotSupportedMultiLocation
			);
			let origin_location = T::AccountIdToMultiLocation::convert(who.clone());
			let mut non_fee_reserve: Option<MultiLocation> = None;
			let asset_len = assets.len();
			for i in 0..asset_len {
				let asset = assets.get(i).ok_or(Error::<T>::AssetIndexNonExistent)?;
				ensure!(
					matches!(asset.fun, Fungibility::Fungible(x) if !x.is_zero()),
					Error::<T>::InvalidAsset
				);
				// `assets` includes fee, the reserve location is decided by non fee asset
				if (fee != *asset && non_fee_reserve.is_none()) || asset_len == 1 {
					non_fee_reserve = T::ReserveProvider::reserve(asset);
				}
				// make sure all non fee assets share the same reserve
				if non_fee_reserve.is_some() {
					ensure!(
						non_fee_reserve == T::ReserveProvider::reserve(asset),
						Error::<T>::DistinctReserveForAssetAndFee
					);
				}
			}
			Ok(())
		}
	}
}