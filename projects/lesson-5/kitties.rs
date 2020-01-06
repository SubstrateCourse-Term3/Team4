use support::{decl_module, decl_storage, ensure, StorageValue, StorageMap, traits::Randomness, dispatch, Parameter};
use sp_runtime::traits::{SimpleArithmetic, Bounded, Member};
use codec::{Encode, Decode};
use runtime_io::hashing::blake2_128;
use system::ensure_signed;
use rstd::result;

pub trait Trait: system::Trait {
	type KittyIndex: Parameter + Member + SimpleArithmetic + Bounded + Default + Copy;
}

#[derive(Encode, Decode)]
pub struct Kitty(pub [u8; 16]);

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct KittyLinkedItem<T: Trait> {
	pub prev: Option<T::KittyIndex>,
	pub next: Option<T::KittyIndex>,
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		/// Stores all the kitties, key is the kitty id / index
		pub Kitties get(fn kitties): map T::KittyIndex => Option<Kitty>;
		/// Stores the total number of kitties. i.e. the next kitty index
		pub KittiesCount get(fn kitties_count): T::KittyIndex;

		pub OwnedKitties get(fn owned_kitties): map (T::AccountId, Option<T::KittyIndex>) => Option<KittyLinkedItem<T>>;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Create a new kitty
		pub fn create(origin) {
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::next_kitty_id()?;

			// Generate a random 128bit value
			let dna = Self::random_value(&sender);

			// Create and store kitty
			let kitty = Kitty(dna);
			Self::insert_kitty(&sender, kitty_id, kitty);
		}

		/// Breed kitties
		pub fn breed(origin, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) {
			let sender = ensure_signed(origin)?;

			Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
		}

		// 作业：实现 transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex)
		// 使用 ensure! 来保证只有主人才有权限调用 transfer
        // 使用 OwnedKitties::append 和 OwnedKitties::remove 来修改小猫的主人
        pub fn transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex){
             
                let sender = ensure_signed(origin)?;

                Self::do_transfer(&sender,  to, kitty_id);	     
                
        }
	}
}

impl<T: Trait> OwnedKitties<T> {
	fn read_head(account: &T::AccountId) -> KittyLinkedItem<T> {
 		Self::read(account, None)
 	}

	fn write_head(account: &T::AccountId, item: KittyLinkedItem<T>) {
 		Self::write(account, None, item);
 	}

	fn read(account: &T::AccountId, key: Option<T::KittyIndex>) -> KittyLinkedItem<T> {
 		<OwnedKitties<T>>::get((&account, key)).unwrap_or_else(|| KittyLinkedItem {
 			prev: None,
 			next: None,
 		})
 	}

	fn write(account: &T::AccountId, key: Option<T::KittyIndex>, item: KittyLinkedItem<T>) {
 		<OwnedKitties<T>>::insert((&account, key), item);
 	}

	pub fn append(account: &T::AccountId, kitty_id: T::KittyIndex) {
		let head = Self::read_head(account);
		let new_head = KittyLinkedItem {
 			prev: Some(kitty_id),
 			next: head.next,
 		};

		Self::write_head(account, new_head);

		let prev = Self::read(account, head.prev);
		let new_prev = KittyLinkedItem {
 			prev: prev.prev,
 			next: Some(kitty_id),
 		};
		Self::write(account, head.prev, new_prev);

		let item = KittyLinkedItem {
 			prev: head.prev,
 			next: None,
 		};
 		Self::write(account, Some(kitty_id), item);
	}

	pub fn remove(account: &T::AccountId, kitty_id: T::KittyIndex) {
		if let Some(item) = <OwnedKitties<T>>::take((&account, Some(kitty_id))) {
			let prev = Self::read(account, item.prev);
			let new_prev = KittyLinkedItem {
 				prev: prev.prev,
 				next: item.next,
 			};

			Self::write(account, item.prev, new_prev);

			let next = Self::read(account, item.next);
 			let new_next = KittyLinkedItem {
 				prev: item.prev,
 				next: next.next,
 			};

  			Self::write(account, item.next, new_next);
		}
    }
    
}


fn  map_bits( num:u8, index:u8 ) -> u8{

    let mut x = 0b00000001;

    let  flag= (num >> index) & x ;
    
    //println!("index={} ,flag is {:b}", index, flag);
    
    return flag;
}

fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
	// 作业：实现combine_dna
	// 伪代码：
	// selector.map_bits(|bit, index| if (bit == 1) { dna1 & (1 << index) } else { dna2 & (1 << index) })
	// 注意 map_bits这个方法不存在。只要能达到同样效果，不局限算法
    // 测试数据：dna1 = 0b11110000, dna2 = 0b11001100, selector = 0b10101010, 返回值 0b11100100

    let mut i =0;
    let mut bit = 0;
    let mut choose;

    let mut new_dna: u8 = 0;

   // println!( "dna1 = {:b}, dna2= {:b}, selector={:b}", dna1,dna2,selector);

    while( i < 8){
        bit = map_bits( selector, i );
        if (bit == 1) 
            {  choose = dna1 & (1<<i)   } 
        else 
             {  choose = dna2 & (1<<i)   }

        //println!( "the index {} ; bit = {}; choose is  {} ", i,  bit, choose);
             
        new_dna |= choose;

             i= i+1;
    }

  	return  new_dna;
}

impl<T: Trait> Module<T> {
	fn random_value(sender: &T::AccountId) -> [u8; 16] {
		let payload = (
			<randomness_collective_flip::Module<T> as Randomness<T::Hash>>::random_seed(),
			&sender,
			<system::Module<T>>::extrinsic_index(),
			<system::Module<T>>::block_number(),
		);
		payload.using_encoded(blake2_128)
	}

	fn next_kitty_id() -> result::Result<T::KittyIndex, &'static str> {
		let kitty_id = Self::kitties_count();
		if kitty_id == T::KittyIndex::max_value() {
			return Err("Kitties count overflow");
		}
		Ok(kitty_id)
	}

	fn insert_owned_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex) {
        // 作业：调用 OwnedKitties::append 完成实现
        <OwnedKitties<T>>::append(&owner,kitty_id );
  	}

     // 第五课作业：实现 transfer(origin, to: T::AccountId, kitty_id: T::KittyIndex)
	// 使用 ensure! 来保证只有主人才有权限调用 transfer
    // 使用 OwnedKitties::append 和 OwnedKitties::remove 来修改小猫的主人
    fn do_transfer(sender: &T::AccountId, to: T::AccountId, kitty_id: T::KittyIndex)-> dispatch::Result{

        println!("Fisher ---1call do_transfer().");
        //1.确保只有主人有权调用transfer  
         ensure!(<OwnedKitties<T>>::get((&sender, Some(kitty_id)))  != None, "Only onwer can transfer the cat!");
        // ensure!(<OwnedKitties<T>>::get((&sender, Some(kitty_id))) .is_some(), "Only onwer can transfer the cat!");
         //ensure!(<OwnedKitties<T>>::exists((&sender, Some(kitty_id))), "Only onwer can transfer the cat!");

            println!("Fisher ---3 Only onwer can transfer the cat!");
            //2.把数据插入到新用户的小猫列表中.
            <OwnedKitties<T>>::append(&to,kitty_id );

            //3.原有用户的小猫列表中，删除被转移小猫的信息.
            <OwnedKitties<T>>::remove(&sender,kitty_id );

         /* 下面这段代码程序执行没有问题，不过前台页面端不会报错，不友好
       if( <OwnedKitties<T>>::get((&sender, Some(kitty_id)))  != None){
            println!("Fisher ---2 begin to transfer the cat.");
            //2.把数据插入到新用户的小猫列表中.
             <OwnedKitties<T>>::append(&to,kitty_id );

            //3.原有用户的小猫列表中，删除被转移小猫的信息.
            <OwnedKitties<T>>::remove(&sender,kitty_id );
       }
       else{
         println!("Fisher ---3 Only onwer can transfer the cat!");
        }
        */

       Ok(())
    }

	fn insert_kitty(owner: &T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty) {
		// Create and store kitty
		<Kitties<T>>::insert(kitty_id, kitty);
		<KittiesCount<T>>::put(kitty_id + 1.into());

		Self::insert_owned_kitty(owner, kitty_id);
	}

	fn do_breed(sender: &T::AccountId, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> dispatch::Result {
		let kitty1 = Self::kitties(kitty_id_1);
		let kitty2 = Self::kitties(kitty_id_2);

		ensure!(kitty1.is_some(), "Invalid kitty_id_1");
		ensure!(kitty2.is_some(), "Invalid kitty_id_2");
		ensure!(kitty_id_1 != kitty_id_2, "Needs different parent");

		let kitty_id = Self::next_kitty_id()?;

		let kitty1_dna = kitty1.unwrap().0;
		let kitty2_dna = kitty2.unwrap().0;

		// Generate a random 128bit value
		let selector = Self::random_value(&sender);
		let mut new_dna = [0u8; 16];

		// Combine parents and selector to create new kitty
		for i in 0..kitty1_dna.len() {
			new_dna[i] = combine_dna(kitty1_dna[i], kitty2_dna[i], selector[i]);
		}

		Self::insert_kitty(sender, kitty_id, Kitty(new_dna));

		Ok(())
	}
}

/// tests for this module
#[cfg(test)]
mod tests {
	use super::*;

	use primitives::H256;
	use support::{impl_outer_origin, assert_ok, parameter_types, weights::Weight};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
	};

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
	fn new_test_ext() -> runtime_io::TestExternalities {
		system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
	}

	#[test]
	fn owned_kitties_can_append_values() {
		new_test_ext().execute_with(|| {
			OwnedKittiesTest::append(&0, 1);

			assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
 				prev: Some(1),
 				next: Some(1),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
 				prev: None,
 				next: None,
 			}));

			OwnedKittiesTest::append(&0, 2);

			assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
 				prev: Some(2),
 				next: Some(1),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
 				prev: None,
 				next: Some(2),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), Some(KittyLinkedItem {
 				prev: Some(1),
 				next: None,
 			}));

			OwnedKittiesTest::append(&0, 3);

  			assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
 				prev: Some(3),
 				next: Some(1),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
 				prev: None,
 				next: Some(2),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), Some(KittyLinkedItem {
 				prev: Some(1),
 				next: Some(3),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(3))), Some(KittyLinkedItem {
 				prev: Some(2),
 				next: None,
 			}));
		});
	}

	#[test]
 	fn owned_kitties_can_remove_values() {
		new_test_ext().execute_with(|| {
			OwnedKittiesTest::append(&0, 1);
 			OwnedKittiesTest::append(&0, 2);
 			OwnedKittiesTest::append(&0, 3);

			OwnedKittiesTest::remove(&0, 2);

			assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
 				prev: Some(3),
 				next: Some(1),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), Some(KittyLinkedItem {
 				prev: None,
 				next: Some(3),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), None);

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(3))), Some(KittyLinkedItem {
 				prev: Some(1),
 				next: None,
 			}));

			OwnedKittiesTest::remove(&0, 1);

  			assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
 				prev: Some(3),
 				next: Some(3),
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), None);

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), None);

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(3))), Some(KittyLinkedItem {
 				prev: None,
 				next: None,
 			}));

			OwnedKittiesTest::remove(&0, 3);

  			assert_eq!(OwnedKittiesTest::get(&(0, None)), Some(KittyLinkedItem {
 				prev: None,
 				next: None,
 			}));

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(1))), None);

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), None);

  			assert_eq!(OwnedKittiesTest::get(&(0, Some(2))), None);
		});
	}
}