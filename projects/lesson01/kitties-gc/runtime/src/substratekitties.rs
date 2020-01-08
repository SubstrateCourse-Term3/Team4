use support::{decl_module, 
    decl_storage, 
    decl_event, 
    StorageValue, 
    StorageMap, 
    ensure, 
    dispatch::Result, 
    traits::{Randomness, Currency, ExistenceRequirement},
    Parameter    
};
use sp_runtime::traits::{ Bounded, SimpleArithmetic, Member};
use system::ensure_signed;
use codec::{Encode, Decode};
use runtime_io::hashing::blake2_128;
use rstd::result;
use crate::linked_item::{LinkedList, LinkedItem};

#[derive(Encode, Decode)]
pub struct Kitty(pub [u8;16]);

///作业:使用 `LinkedList` 重构 OwnedKitties 和 KittyLinkedItem 
type KittyLinkedItem<T> = LinkedItem<<T as Trait>::KittyIndex>;
type OwnedKittiesList<T> = LinkedList<OwnedKitties<T>, <T as system::Trait>::AccountId, <T as Trait>::KittyIndex>;

pub trait Trait: system::Trait {
    type KittyIndex: Parameter + Member + SimpleArithmetic + Bounded + Copy + Default;
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
    type Randomness: Randomness<Self::Hash>;
}

type BalanceOf<T> =  <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
//测试测试数据dna1 = 0b11110000 dna2 = 0b11001100 selector=0b10101010 ,返回值0b11100100
fn combine_dna(dna1: u8,dna2: u8,selector: u8) -> u8{
    ((selector & dna1) | (!selector & dna2))
}

decl_event! {
    pub enum Event<T>
    where 
        <T as system::Trait>::AccountId,
        <T as Trait>::KittyIndex,
        Balance = BalanceOf<T>
    {
        /// A kitty is created. (owner, kitty_id)
        Created(AccountId, KittyIndex),
        /// A kitty is transferred. (from, to, kitty_id)
        Transferred(AccountId, AccountId, KittyIndex),
        /// A kitty is available for sale. (owner, kitty_id, price)
        Ask(AccountId, KittyIndex, Option<Balance>),
        /// A kitty is sold. (from, to, kitty_id, price)
        Sold(AccountId, AccountId, KittyIndex, Balance),
    }
}
decl_storage! {
    trait Store for  Module<T: Trait> as KittyStorage {
        
        pub Kitties get(fn kitties): map T::KittyIndex => Option<Kitty>;
        
        pub KittyOwner get(fn kitty_owner): map T::KittyIndex => Option<T::AccountId>;

        pub KittiesCount get(fn kitties_count): T::KittyIndex;

        pub OwnedKitties get(fn owned_kitties): map (T::AccountId, Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;

        pub KittyPrices get(fn kitty_price): map T::KittyIndex => Option<BalanceOf<T>>;
    }
}
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin{
        fn deposit_event() = default;   
        fn create_kitties(origin) {
            let sender = ensure_signed(origin)?;
            // let random_seed = <randomness_collective_flip::Module<T> as Randomness<T::Hash>>::random_seed();
            let random_seed = T::Randomness::random_seed();
            let dna = Self::random_value(&sender, &random_seed);

            let kitty = Kitty(dna);

            let kitty_id = Self::next_kitty_id()?;
            
            Self::insert_kitty(sender.clone(), kitty_id, kitty);
          
            Self::deposit_event(RawEvent::Created(sender, kitty_id));
        }
        //繁殖小猫
        fn breed_kitty(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex){
            let sender = ensure_signed(origin)?;
            
            let new_kitty_id = Self::do_breed(sender.clone(),kitty_id_1,kitty_id_2)?;

            Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
        }
        //设置价格
        fn ask(origin, kitty_id: T::KittyIndex, price: Option<BalanceOf<T>>){
            let sender = ensure_signed(origin)?;

            ensure!(<Kitties<T>>::exists(kitty_id), "This kitty does not exist");

            let owner = Self::kitty_owner(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner == sender, "You do not own this cat");


            if let Some(ref price) = price {
                <KittyPrices<T>>::insert(kitty_id, price);
            }else {
                <KittyPrices<T>>::remove(kitty_id);
            }
///作业1:添加Ask和Sold Event
            Self::deposit_event(RawEvent::Ask(sender, kitty_id, price));
        }
        //购买小猫
        fn buy(origin, kitty_id: T::KittyIndex, price: BalanceOf<T>){
            let sender = ensure_signed(origin)?;

            let owner = Self::kitty_owner(kitty_id);
            ensure! (owner.is_some(), "Kitty does not exit.");
            let owner = owner.unwrap();
            let kitty_price = Self::kitty_price(kitty_id);
            ensure!(kitty_price.is_some(), "Kitty not for sale");

            let kitty_price = kitty_price.unwrap();
            ensure!(price >= kitty_price, "price is too low");

            //<balances::Module<T> as Currency<T::AccountId>>::transfer(&sender, &owner, kitty_price, ExistenceRequirement::KeepAlive)?;
            T::Currency::transfer(&sender, &owner, kitty_price, ExistenceRequirement::KeepAlive)?;

            <KittyPrices<T>>::remove(kitty_id);

            Self::do_transfer(&owner, &sender, kitty_id);
///作业1:添加Ask和Sold Event
            Self::deposit_event(RawEvent::Sold(owner, sender, kitty_id, kitty_price));
        }
        //转移猫
        fn transfer_kitty(origin, to: T::AccountId, kitty_id: T::KittyIndex) -> Result{
            let sender = ensure_signed(origin)?;

            let owner = Self::kitty_owner(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner == sender, "You do not own this kitty"); 
            Self::do_transfer(&sender, &to, kitty_id);
            Ok(())
        }
    }
}

impl<T:Trait> Module<T>{
//作业2:完成'transfer'
    fn do_transfer (from: &T::AccountId, to: &T::AccountId, kitty_id: T::KittyIndex){
        //从原有账户删除
        <OwnedKittiesList<T>>::remove(from,kitty_id);
        //附加到新的账户
        <OwnedKittiesList<T>>::append(to,kitty_id);
        //修改小猫的主人
        <KittyOwner<T>>::insert(kitty_id,to);
    }
    //生成随机数
    fn random_value(sender: &T::AccountId,random_seed: &T::Hash) -> [u8;16]{
        let payload = (
            random_seed,
            sender.clone(),
            <system::Module<T>>::extrinsic_index(),
            <system::Module<T>>::block_number(),
        );
        payload.using_encoded(blake2_128)
    }
    //下一只猫的id
    fn next_kitty_id() -> result::Result<T::KittyIndex, &'static str>{
        let kitty_id = Self::kitties_count();
        //检测是否溢出
        if kitty_id == T::KittyIndex::max_value(){
            return Err("kitty count overflow");
        }
        Ok(kitty_id)
    }
///作业3:完成insert_owned_kitty
    fn insert_owned_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex){
        //使用链表方式给用户添加
        <OwnedKittiesList<T>>::append(&owner, kitty_id);
    }
    //生成新的小猫并做关联
    fn insert_kitty(owner: T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty){
        <Kitties<T>>::insert(kitty_id, kitty);
        <KittyOwner<T>>::insert(kitty_id, owner.clone());
        <KittiesCount<T>>::put(kitty_id + 1.into());
        //store the ownership information
        //用户拥有小猫的索引
        // let user_kitty_id =Self::owned_kitties_count(owner.clone());
        // //根据用户id和下属kitty的索引，找到kitty在整个数组的编号
        // <OwnedKitties<T>>::insert((owner.clone(),user_kitty_id),kitty_id);
        // //更新用户对应kitty的索引
        // <OwnedKittiesCount<T>>::insert(owner.clone(),user_kitty_id+1);
        Self::insert_owned_kitty(&owner, kitty_id);
    }
    //繁殖小猫帮助函数
    fn do_breed(sender: T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> result::Result<T::KittyIndex, &'static str> {
        let kitty1 = Self::kitties(kitty_id_1);
        let kitty2 = Self::kitties(kitty_id_2);

        ensure!(kitty1.is_some(), "Invalid kitty_id_1");
        ensure!(kitty2.is_some(), "Invalid kitty_id_2");
        ensure!(kitty_id_1 != kitty_id_2, "Need different parent");

        let new_kitty_id = Self::next_kitty_id()?;
        let kitty1_dna = kitty1.unwrap().0;
        let kitty2_dna = kitty2.unwrap().0;
        
        let random_seed = <randomness_collective_flip::Module<T> as Randomness<T::Hash>>::random_seed();
        let selector = Self::random_value(&sender,&random_seed);
        let mut new_dna=[0u8;16];
        for i in 0..kitty1_dna.len() {
            new_dna[i] = combine_dna(kitty1_dna[i],kitty2_dna[i],selector[i]);
        }
        let new_kitty = Kitty(new_dna);
        Self::insert_kitty(sender.clone(), new_kitty_id, new_kitty);
        Ok(new_kitty_id)
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use primitives::H256;
	use support::{impl_outer_origin, assert_ok, parameter_types, weights::Weight,impl_outer_event};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
    };
    use crate::linked_item::{LinkedList,LinkedItem};
	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the module, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq, Debug)]
	pub struct Test;
	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub const MaximumBlockWeight: Weight = 1024;
		pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
        pub const ExistentialDeposit: u64 = 0;
		pub const TransferFee: u64 = 0;
        pub const CreationFee: u64 = 0;
    }

	impl system::Trait for Test {
		type Origin = Origin;
		type Call = ();
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
    }
    impl balances::Trait for Test {
		type Balance = u64;
		type OnFreeBalanceZero = ();
		type OnNewAccount = ();
		type Event = ();
		type TransferPayment = ();
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type TransferFee = TransferFee;
		type CreationFee = CreationFee;
	}
	impl Trait for Test {
        type KittyIndex = u32;
        type Event = ();
        type Currency = Balances;
        type Randomness = Randomness;
	}
    type OwnedKittiesTest = OwnedKittiesList<Test>;
    type Balances = balances::Module<Test>;
    type Randomness = randomness_collective_flip::Module<Test>;
	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn owned_kitties_can_append_values() {
		new_test_ext().execute_with(|| {
            
            OwnedKittiesTest::append(&0, 1);

            assert_eq! (OwnedKittiesTest::read(&0, None), LinkedItem{
                prev: Some(1),
                next: Some(1),
            });            
            assert_eq! (OwnedKittiesTest::read(&0, Some(1)), LinkedItem{
                prev: None,
                next: None,
            });

            OwnedKittiesTest::append(&0, 2);

            assert_eq! (OwnedKittiesTest::read(&0, None), LinkedItem{
                prev: Some(2),
                next: Some(1),
            });
            assert_eq! (OwnedKittiesTest::read(&0, Some(1)), LinkedItem{
                prev: None,
                next: Some(2),
            });      
            assert_eq! (OwnedKittiesTest::read(&0, Some(2)), LinkedItem{
                prev: Some(1),
                next: None,
            });      
            
            OwnedKittiesTest::append(&0, 3);

            assert_eq! (OwnedKittiesTest::read(&0, None), LinkedItem{
                prev: Some(3),
                next: Some(1),
            });
            assert_eq! (OwnedKittiesTest::read(&0, Some(1)), LinkedItem{
                prev: None,
                next: Some(2),
            });     
            // assert_eq! (OwnedKittiesTest::read(&0, Some(2)), LinkedItem{
            //     prev: Some(1),
            //     next: None,
            // });  
            // assert_eq! (OwnedKittiesTest::read(&0, Some(3)), LinkedItem{
            //     prev: Some(1),
            //     next: Some(3),
            // }); 
            // assert_eq! (OwnedKittiesTest::read(&0, Some(3)), LinkedItem{
            //     prev: Some(2),
            //     next: None,
            // });  
	    });
	}
}