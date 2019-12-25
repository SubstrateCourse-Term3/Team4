//加密猫设计V2版本

/// 作业1. 设计加密猫模块V2
/// 	需求：
///    	1). 繁殖小猫
///    	2). 选择两只现有的猫作为父母
///    	3). 小猫必须继承父母的基因
///    	4). 同样的父母生出来的小猫不能相同

	//数据结构
		struct Kitty {
			id: Hash,
			dna: [u8;16],//dna
			price: Balance,//价格
			gen: u32,//第N代
		}
	//存储定义
		//存储猫的map
	    pub Kitties get(kitty): map u32 => Kitty;
	    //猫的数量
	    pub KittiesCount get(kitties_count): u32;
	//繁殖小猫
		fn breed_kitty(origin, id1:u32, id2:u32) -> Result{

			let sender = ensure_signed(origin)?;

			let kitty1 = Self::kitty(id1);
			let kitty2 = Self::kitty(id2);

			//dna繁殖算法
			let random_seed = (
			                <system::Module<T>>::random_seed(),
			                &sender,
			                <system::Module<T>>::extrinsic_index(),
			                <system::Module<T>>::block_number(),
	        ).using_encoded(blake2_128);

			let final_dna = kitty1.dna;
			for (i,(kitty2_dna_elem,r)) in kitty2.dna.iter().zip(random_seed.iter()).enumerate() {
				if r % 2 == 0 {
					final_dna[i] = *kitty2_dna_elem;
				}
				
			}
			let new_kitty = {
				id:random_seed,
				dna:final_dna,
				price:0,
				gen:max(kitty1.gen,kitty2.gen)+1,
			}
			//生成新的小猫
			let count = Self::kitties_count();
/// 2. create 这里面的kitties_count有溢出的可能性，修复这个问题
      //加1检测是否溢出，异常返回提示
      let new_count = count.checked_add(1).ok_or("Overflow adding a new kitty")?;
      Kitties::insert(new_count,new_kitty);
      KittiesCount::put(new_count);

			Ok(())
		}
