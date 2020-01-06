use support::{decl_module, 
    decl_storage, 
    decl_event, 
    StorageValue, 
    StorageMap, 
    ensure, 
    dispatch::Result, 
    traits::Randomness,
    Parameter    
};
use sp_runtime::traits::{ Bounded, SimpleArithmetic, Member};
use system::ensure_signed;
use codec::{Encode, Decode};
use runtime_io::hashing::blake2_128;
use rstd::result;

#[derive(Encode, Decode)]
// pub struct Kitty(pub [u8;16]);
pub struct Kitty{
    dna: [u8;16],
    price: u128,
}

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct KittyLinkedItem<T: Trait>{
    pub prev: Option<T::KittyIndex>,
    pub next: Option<T::KittyIndex>,
}

pub trait Trait: balances::Trait {
    type KittyIndex: Parameter + Member + SimpleArithmetic + Bounded + Copy + Default;
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

//测试测试数据dna1 = 0b11110000 dna2 = 0b11001100 selector=0b10101010 ,返回值0b11100100
fn combine_dna(dna1: u8,dna2: u8,selector: u8) -> u8{
    //亮的算法
    ((selector & dna1) | (!selector & dna2))
    //原算法
    // let mut result_dna = 0;
    // let mut tmp_dna;
    // for i in 0..8 {
    //     let mut selector_bit = 1;
    //     selector_bit = selector_bit << i;
    //     if selector & selector_bit == 1{
    //         tmp_dna = dna1 & selector_bit;
    //     }else{
    //         tmp_dna = dna2 & selector_bit;
    //     }
    //     result_dna |= tmp_dna;
    // }
    // return result_dna;
}
impl<T: Trait> OwnedKitties<T>{
    fn read_head(account: &T::AccountId) -> KittyLinkedItem<T>{
        Self::read(account,None)
    }
    fn write_head(account: &T::AccountId, item: KittyLinkedItem<T>){
        Self::write(account, None, item)
    }
    fn read(account: &T::AccountId, key: Option<T::KittyIndex>) -> KittyLinkedItem<T>{
        <OwnedKitties<T>>::get(&(account.clone(), key)).unwrap_or_else(|| KittyLinkedItem{
            prev:None,
            next:None,
        })
    }
    fn write(account: &T::AccountId, key: Option<T::KittyIndex>, item: KittyLinkedItem<T>){
        <OwnedKitties<T>>::insert(&(account.clone(), key), item);
    }
    pub fn append(account: &T::AccountId, kitty_id: T::KittyIndex){
        let head  = Self::read_head(account);
        let new_head = KittyLinkedItem{
            prev: Some(kitty_id),
            next: head.next
        };
        Self::write_head(account, new_head);

        let prev = Self::read(account, head.prev);
        let new_prev = KittyLinkedItem{
            prev: prev.prev,
            next: Some(kitty_id)
        };
        Self::write(account, head.prev, new_prev);

        let item = KittyLinkedItem{
            prev: head.prev,
            next: None,
        };
        Self::write(account, Some(kitty_id), item);
    }
    pub fn remove(account: &T::AccountId, kitty_id: T::KittyIndex){

        if let Some(item) = <OwnedKitties<T>>::take(&(account.clone(), Some(kitty_id))){
            let prev = Self::read(account, item.prev);
            let new_prev = KittyLinkedItem{
                prev: prev.prev,
                next: item.next,
            };
            Self::write(account, item.prev, new_prev);

            let next = Self::read(account, item.next);
            let new_next = KittyLinkedItem {
                prev: item.prev,
                next: next.next
            };
            Self::write(account, item.next, new_next);
        }
    }
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
        
        pub Kitties get(fn kitties): map T::KittyIndex => Option<Kitty>;
        pub KittyOwner get(fn kitty_owner): map T::KittyIndex => Option<T::AccountId>;
        pub KittiesCount get(fn kitties_count): T::KittyIndex;

        //Get kitty Id by Account Id and user kitty index
        //pub OwnedKitties get(owned_kitties): map (T::AccountId, KittyIndex) => KittyIndex;
        //Get number of kitties by account Id
        //pub OwnedKittiesCount get(owned_kitties_count): map T::AccountId => KittyIndex;
        pub OwnedKitties get(fn owned_kitties): map (T::AccountId, Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;
    }
}
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin{
        fn deposit_event() = default;   
        fn create_kitties(origin) -> Result {
            let sender = ensure_signed(origin)?;
            let random_seed = <randomness_collective_flip::Module<T> as Randomness<T::Hash>>::random_seed();
            let dna = Self::random_value(&sender, &random_seed);

            let kitty = Kitty{
                dna: dna,
                price: 0,
            };

            let kitty_id = Self::next_kitty_id()?;
            
            Self::insert_kitty(sender.clone(), kitty_id, kitty);
          
            Self::deposit_event(RawEvent::Created(sender, random_seed));
            Ok(())
        }
///作业1-1:给自己的小猫设置价格
        //设置价格
        fn set_pirce(origin, kitty_id: T::KittyIndex, price: u128){
            let sender = ensure_signed(origin)?;

            ensure!(<Kitties<T>>::exists(kitty_id), "This kitty does not exist");

            let owner = Self::kitty_owner(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner == sender, "You do not own this cat");

            let mut kitty = Self::kitties(kitty_id).unwrap();
            kitty.price = price;

            <Kitties<T>>::insert(kitty_id, kitty);
        }
///作业1-2:购买其他人的小猫
        fn buy_kitty(origin, kitty_id: T::KittyIndex){
            let sender = ensure_signed(origin)?;
            //检测存在
            ensure!(<Kitties<T>>::exists(kitty_id), "This kitty does not exist");
            //获取主人
            let owner = Self::kitty_owner(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner != sender, "You can't buy your own cat");

            let kitty = Self::kitties(kitty_id).unwrap();

            let kitty_price = kitty.price;
            ensure!(kitty_price == 0, "The kitty you want to buy is not for sale");

            //伪代码:购买将钱从sender转给owner
            //transfer_cash(&sender, &owner, kitty_price)

            //将小猫从owner转给sender
            Self::transfer(&owner, &sender, kitty_id);
        }
        //繁殖小猫
        fn breed_kitty(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> Result {
            let sender = ensure_signed(origin)?;
            Self::do_breed(sender,kitty_id_1,kitty_id_2)?;
            Ok(())
        }
        //转移猫
        fn transfer_kitty(origin, to: T::AccountId, kitty_id: T::KittyIndex) -> Result{
            let sender = ensure_signed(origin)?;
            let owner = Self::kitty_owner(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner == sender, "You do not own this kitty");
            Self::transfer(&sender, &to, kitty_id);
            //根据AccountId和user_kitty_id获取kittyid
            // let kittyid = Self::owned_kitties((sender.clone(),user_kitty_id));
            // //更新数组
            // <OwnedKitties<T>>::remove((sender.clone(),user_kitty_id));
            // <OwnedKitties<T>>::insert((to.clone(),user_kitty_id),kittyid);

            // //更新数量
            // let owned_kitties_count_from = Self::owned_kitties_count(sender.clone()).checked_add(1).ok_or("Transfer kitty overflow")?;
            // let owned_kitties_count_to = Self::owned_kitties_count(to.clone()).checked_sub(1).ok_or("Transfer kitty underflow")?;
            // <OwnedKittiesCount<T>>::insert(to.clone(),owned_kitties_count_to);
            // <OwnedKittiesCount<T>>::insert(sender.clone(),owned_kitties_count_from);
            Ok(())
        }
    }
}

impl<T:Trait> Module<T>{
//作业2:完成'transfer'
    fn transfer (from: &T::AccountId, to: &T::AccountId, kitty_id: T::KittyIndex){
        <KittyOwner<T>>::insert(kitty_id,to);
        //附加到新的账户
        <OwnedKitties<T>>::append(to,kitty_id);
        //从原有账户删除
        <OwnedKitties<T>>::remove(from,kitty_id);
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
        <OwnedKitties<T>>::append(&owner, kitty_id);
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
    fn do_breed(sender: T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> Result {
        let kitty1 = Self::kitties(kitty_id_1);
        let kitty2 = Self::kitties(kitty_id_2);

        ensure!(kitty1.is_some(), "Invalid kitty_id_1");
        ensure!(kitty2.is_some(), "Invalid kitty_id_2");
        ensure!(kitty_id_1 != kitty_id_2, "Need different parent");

        let kitty_id = Self::next_kitty_id()?;
        let kitty1_dna = kitty1.unwrap().dna;
        let kitty2_dna = kitty2.unwrap().dna;
        
        let random_seed = <randomness_collective_flip::Module<T> as Randomness<T::Hash>>::random_seed();
        let selector = Self::random_value(&sender,&random_seed);
        let mut new_dna=[0u8;16];
        for i in 0..kitty1_dna.len() {
            new_dna[i] = combine_dna(kitty1_dna[i],kitty2_dna[i],selector[i]);
        }
        let new_kitty = Kitty{
            dna: new_dna,
            price: 0,
        };
        Self::insert_kitty(sender.clone(), kitty_id, new_kitty);
        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use support::{impl_outer_origin, assert_ok, parameter_types};
	use sr_primitives::{traits::{BlakeTwo256, IdentityLookup}, testing::Header};
	use sr_primitives::weights::Weight;
	use sr_primitives::Perbill;

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
		type WeightMultiplierUpdate = ();
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type MaximumBlockWeight = MaximumBlockWeight;
		type MaximumBlockLength = MaximumBlockLength;
		type AvailableBlockRatio = AvailableBlockRatio;
		type Version = ();
	}
	impl Trait for Test {
        type KittyIndex = u32;
	}
    type OwnedKittiesTest = OwnedKitties<Test>;
	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn owned_kitties_can_append_values() {
		with_externalities(&mut new_test_ext(), || {
            
            OwnedKittiesTest::append(&0, 1);

            assert_eq! (OwnedKittiesTest::get(&0, None), Some(KittyLinkedItem{
                prev: Some(1),
                next: Some(1),
            }));            
            assert_eq! (OwnedKittiesTest::get(&0, Some(1)), Some(KittyLinkedItem{
                prev: None,
                next: None,
            }));

            OwnedKittiesTest::append(&0, 2);

            assert_eq! (OwnedKittiesTest::get(&0, None), Some(KittyLinkedItem{
                prev: Some(2),
                next: Some(1),
            }));
            assert_eq! (OwnedKittiesTest::get(&0, Some(1)), Some(KittyLinkedItem{
                prev: None,
                next: Some(2),
            }));      
            assert_eq! (OwnedKittiesTest::get(&0, Some(2)), Some(KittyLinkedItem{
                prev: Some(1),
                next: None,
            }));      
            
            OwnedKittiesTest::append(&0, 3);

            assert_eq! (OwnedKittiesTest::get(&0, None), Some(KittyLinkedItem{
                prev: Some(3),
                next: Some(1),
            }));
            assert_eq! (OwnedKittiesTest::get(&0, Some(1)), Some(KittyLinkedItem{
                prev: None,
                next: Some(2),
            }));     
            assert_eq! (OwnedKittiesTest::get(&0, Some(2)), Some(KittyLinkedItem{
                prev: Some(1),
                next: None,
            }));  
            assert_eq! (OwnedKittiesTest::get(&0, Some(3)), Some(KittyLinkedItem{
                prev: Some(1),
                next: Some(3),
            })); 
            assert_eq! (OwnedKittiesTest::get(&0, Some(3)), Some(KittyLinkedItem{
                prev: Some(2),
                next: None,
            }));  
		});
	}
}