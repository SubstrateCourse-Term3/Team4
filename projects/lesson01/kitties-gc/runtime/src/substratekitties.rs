use support::{decl_module, decl_storage, decl_event, StorageValue, StorageMap, dispatch::Result};

use system::ensure_signed;
use codec::{Encode, Decode};
use runtime_io::blake2_128;

// pub trait Trait: system::Trait{}

#[derive(Encode, Decode, Default)]
pub struct Kitty(pub [u8;16]);


pub trait Trait: system::Trait {
	type Event : From<Event<Self>> + Into<<Self as system::Trait>::Event>;

}
decl_event! {
    pub enum Event<T>
    where 
        <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash
    {
        Created(AccountId,Hash),
    }
}
decl_storage! {
    trait Store for  Module<T: Trait> as KittyStorage {
        
        pub Kitties get(kitties): map u32 => Kitty;
        pub KittiesCount get(kitties_count): u32;


        // KittyOwner get(owner_of): map u32 => Option<T::AccountId>;
        // OwnedKitty get(kitty_of_owner): map T::AccountId => u32;

        // //人员列表
        // PeopleArray get(person): map u32 => T::AccountId;
        // //人员数量
        // PeopleCount get(num_of_people): u32;
    }
}
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin{

        fn deposit_event() = default;

        fn create_kitties(origin) -> Result {
            let sender = ensure_signed(origin)?;
            let random_seed = <system::Module<T>>::random_seed();
            let payload = (
                // <randomness_collective_flip::Module<T> as Randomness<T::Hash>>::random_seed(),
                random_seed,
                &sender,
                <system::Module<T>>::extrinsic_index(),
                <system::Module<T>>::block_number(),
            );
            let dna = payload.using_encoded(blake2_128);
            let kitty = Kitty(dna);
            let count = Self::kitties_count();
            //检测是否溢出
            let new_count = count.checked_add(1).ok_or("Overflow adding a new kitty")?;
            Kitties::insert(count,kitty);
            KittiesCount::put(new_count);

            Self::deposit_event(RawEvent::Created(sender,random_seed));
            Ok(())
        }

        // fn add_person(origin, new_person: T::AccountId) -> Result {
        //     let sender = ensure_signed(origin)?;
        //     let people_count = Self::num_of_people();
        //     let new_people_count = people_count.checked_add(1).ok_or("Overflow adding a person")?;

        //     <PeopleArray<T>>::insert(people_count,new_person);
        //     PeopleCount::put(new_people_count);
        //     Ok(())
        // }
        
    }
}
