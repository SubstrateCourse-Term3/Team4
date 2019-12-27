use support::{decl_module,decl_storage};
use codec::{Encode,Decode};
//#[derive(Encode,Decode,Default)]
#[derive(Encode, Decode, Default)]
pub struct Kitty(u128);

pub trait Trait:system::Trait{
}
//add data struct


//save kitty
decl_storage!{
    trait Store for Module<T:Trait> as Kitties{
    pub Kitties get(kitties):map u32=>Kitty;
    pub KittiesCount get(fn kitties_count):u32;
    }
}

decl_module!{
  pub struct Module<T:Trait> for enum Call where origin:T::Origin{

  }
}