use support::{decl_module, decl_storage, decl_event, StorageValue, StorageMap, ensure, dispatch::Result};

use system::ensure_signed;
use codec::{Encode, Decode};
use runtime_io::blake2_128;
use rstd::result;

// pub trait Trait: system::Trait{}
#[derive(Encode, Decode)]
pub struct Kitty(pub [u8;16]);

pub trait Trait: system::Trait {
	type Event : From<Event<Self>> + Into<<Self as system::Trait>::Event>;

}
///作业3:完成combine_dna函数 
//测试测试数据dna1 = 0b11110000 dna2 = 0b11001100 selector=0b10101010 ,返回值0b11100100
fn combine_dna(dna1: u8,dna2: u8,selector: u8) -> u8{
    let mut selector_bit = 1;
    let mut result_dna = 0;
    let mut tmp_dna;
    for i in 0..8 {
        selector_bit = selector_bit << i;
        if selector & selector_bit == 1{
            tmp_dna = dna1 & selector_bit;
        }else{
            tmp_dna = dna2 & selector_bit;
        }
        result_dna |= tmp_dna;
    }
    return result_dna;
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
        
        pub Kitties get(kitties): map u32 => Option<Kitty>;
        pub KittiesCount get(kitties_count): u32;

        //Get kitty Id by Account Id and user kitty index
        pub OwnedKitties get(owned_kitties): map (T::AccountId, u32) => u32;
        //Get number of kitties by account Id
        pub OwnedKittiesCount get(owned_kitties_count): map T::AccountId => u32;
        //Get kitty index by kittyid
        // pub OwnedKittiesIndex get(owned_kitties_index): map u32 => u32;
    }
}
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin{

        fn deposit_event() = default;
///作业2:使用帮助函数重构了create_kitties函数        
        fn create_kitties(origin) -> Result {
            let sender = ensure_signed(origin)?;
            
            let random_seed = <system::Module<T>>::random_seed();
            
            let dna = Self::random_value(&sender, &random_seed);

            let kitty = Kitty(dna);

            let kitty_id = Self::next_kitty_id()?;
            
            Self::insert_kitty(sender.clone(), kitty_id, kitty);
          
            Self::deposit_event(RawEvent::Created(sender, random_seed));
            Ok(())
        }
        //繁殖小猫
        fn breed_kitty(origin,kitty_id_1:u32,kitty_id_2:u32) -> Result {
            let sender = ensure_signed(origin)?;
            Self::do_breed(sender,kitty_id_1,kitty_id_2)?;
            Ok(())
        }
///作业1:完成转移猫操作(依据原有数据结构，并没有使用链表数据结构)
        fn transfer_kitty(origin, to: T::AccountId, user_kitty_id: u32) -> Result{
            
            let sender = ensure_signed(origin)?;
            //根据AccountId和user_kitty_id获取kittyid
            let kittyid = Self::owned_kitties((sender.clone(),user_kitty_id));
            //更新数组
            <OwnedKitties<T>>::remove((sender.clone(),user_kitty_id));
            <OwnedKitties<T>>::insert((to.clone(),user_kitty_id),kittyid);

            //更新数量
            let owned_kitties_count_from = Self::owned_kitties_count(sender.clone()).checked_add(1).ok_or("Transfer kitty overflow")?;
            let owned_kitties_count_to = Self::owned_kitties_count(to.clone()).checked_sub(1).ok_or("Transfer kitty underflow")?;
            <OwnedKittiesCount<T>>::insert(to.clone(),owned_kitties_count_to);
            <OwnedKittiesCount<T>>::insert(sender.clone(),owned_kitties_count_from);
            Ok(())
        }
    }
}

impl<T:Trait> Module<T>{
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
    fn next_kitty_id() -> result::Result<u32,&'static str>{
        let kitty_id = Self::kitties_count();
        //检测是否溢出
        if kitty_id == u32::max_value(){
            return Err("kitty count overflow");
        }
        Ok(kitty_id)
    }
    //生成新的小猫并做关联
    fn insert_kitty(owner: T::AccountId, kitty_id: u32, kitty: Kitty){
        Kitties::insert(kitty_id,kitty);
        KittiesCount::put(kitty_id+1);
        //store the ownership information
        //用户拥有小猫的索引
        let user_kitty_id =Self::owned_kitties_count(owner.clone());
        //根据用户id和下属kitty的索引，找到kitty在整个数组的编号
        <OwnedKitties<T>>::insert((owner.clone(),user_kitty_id),kitty_id);
        //更新用户对应kitty的索引
        <OwnedKittiesCount<T>>::insert(owner.clone(),user_kitty_id+1);
    }
    //繁殖小猫封装，便于调试
    fn do_breed(sender: T::AccountId, kitty_id_1: u32, kitty_id_2: u32) -> Result {
        let kitty1 = Self::kitties(kitty_id_1);
        let kitty2 = Self::kitties(kitty_id_2);

        ensure!(kitty1.is_some(),"Invalid kitty_id_1");
        ensure!(kitty2.is_some(),"Invalid kitty_id_2");
        ensure!(kitty_id_1 != kitty_id_2,"Need different parent");

        let kitty_id = Self::next_kitty_id()?;
        let kitty1_dna = kitty1.unwrap().0;
        let kitty2_dna = kitty2.unwrap().0;
        
        let random_seed = <system::Module<T>>::random_seed();
        let selector = Self::random_value(&sender,&random_seed);
        let mut new_dna=[0u8;16];
        for i in 0..kitty1_dna.len() {
            new_dna[i] = combine_dna(kitty1_dna[i],kitty2_dna[i],selector[i]);
        }
        Self::insert_kitty(sender.clone(),kitty_id,Kitty(new_dna));
        Ok(())
    }
}
