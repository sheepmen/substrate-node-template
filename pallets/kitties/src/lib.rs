#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure, fail, StorageValue, StorageMap, traits::Randomness};
use sp_io::hashing::blake2_128;
use frame_system::ensure_signed;
use sp_runtime::DispatchError;


type KittyIndex = u32;
#[derive(Encode, Decode, Debug)]
pub struct Kitty(pub [u8; 16]);

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
        pub Kitties get(fn kitties): map hasher(blake2_128_concat) KittyIndex => Option<Kitty>;
        pub KittiesCount get(fn kitties_count): KittyIndex;
        pub KittyOwners get(fn kitty_owner):
            map
                hasher(blake2_128_concat) KittyIndex
                => Option<T::AccountId>;
        pub OwnerKitties get(fn owner_kitty):
            double_map
                hasher(blake2_128_concat) T::AccountId,
                hasher(blake2_128_concat) KittyIndex
                => KittyIndex;
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
	    KittiesCountOverflow,
	    InvalidKittyId,
	    RequireDifferentParent,
	    KittyIdNotExist,
	    NotKittyOwner,
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
	    Created(AccountId, KittyIndex),
	    Transferred(AccountId, AccountId, KittyIndex),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
	    type Error = Error<T>;
		fn deposit_event() = default;

	    #[weight = 0]
		pub fn create(origin) {
            let sender = ensure_signed(origin)?;
            let kitty_id = Self::next_kitty_id()?;
            let dna = Self::random_value(&sender);
            let kitty = Kitty(dna);
            Self::insert_kitty(&sender, kitty_id, kitty);
            Self::deposit_event(RawEvent::Created(sender, kitty_id));
		}

		#[weight = 0]
		pub fn transfer(origin, to: T::AccountId, kitty_id: KittyIndex) {
            let sender = ensure_signed(origin)?;

            // // 检查kitty是否存在
			// ensure!(Kitties::contains_key(&kitty_id), Error::<T>::KittyIdNotExist);
			// // 检查是否为kitty的拥有者
			// let owner = KittyOwners::<T>::get(&kitty_id).unwrap();
			// ensure!(owner == sender, Error::<T>::NotKittyOwner);

            match KittyOwners::<T>::get(&kitty_id) {
                Some(owner) => ensure!(owner == sender, Error::<T>::NotKittyOwner),
                None => fail!(Error::<T>::KittyIdNotExist)
            }
            <OwnerKitties<T>>::remove(&sender, kitty_id);
            <OwnerKitties<T>>::insert(to.clone(), kitty_id, kitty_id);
            <KittyOwners<T>>::insert(kitty_id, to.clone());
            Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
		}

        #[weight = 0]
		pub fn breed(origin, kitty_id_1: KittyIndex, kitty_id_2: KittyIndex) {
		    let sender = ensure_signed(origin)?;
            let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
            Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
		}
	}

}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    (selector & dna1) | (!selector & dna2)
}

impl<T: Trait> Module<T> {
    fn do_breed(sender: &T::AccountId, kitty_id_1: KittyIndex, kitty_id_2: KittyIndex) -> sp_std::result::Result<KittyIndex, DispatchError> {
        let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
        let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;

        ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RequireDifferentParent);

        let kitty_id = Self::next_kitty_id()?;
        let kitty1_dna = kitty1.0;
        let kitty2_dna = kitty2.0;
        let selector = Self::random_value(&sender);
        let mut new_dna = [0u8; 16];
        for i in 0..kitty1_dna.len() {
            new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
        }
        Self::insert_kitty(&sender, kitty_id, Kitty(new_dna));
        Ok(kitty_id)
    }

    fn insert_kitty(owner: &T::AccountId, kitty_id: KittyIndex, kitty: Kitty) {
        Kitties::insert(kitty_id, kitty);
        KittiesCount::put(kitty_id + 1);
        <OwnerKitties<T>>::insert(owner.clone(), kitty_id, kitty_id);
        <KittyOwners<T>>::insert(kitty_id, owner);
    }

    fn next_kitty_id() -> sp_std::result::Result<KittyIndex, DispatchError> {
        let kitty_id = Self::kitties_count();
        if kitty_id == KittyIndex::max_value() {
            return Err(Error::<T>::KittiesCountOverflow.into());
        }
        Ok(kitty_id)
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = (
            T::Randomness::random_seed(),
            &sender,
            <frame_system::Module<T>>::extrinsic_index()
        );
        payload.using_encoded(blake2_128)
    }
}