//数据结构
	struct kitty {
		id: Accountid,
		dna: hash,
		price: Balance,
	}

//存储定义
	let mut map = Map::new();
	//每个用户账户对应多只猫
	map Accountid => vec<kitty>

//可调用函数
	//创建用户账户
	fn add_account() -> Result {
	}
	fn create_kitty(self) -> Result {
	}
	//设置猫的价格
	fn set_price(self,id) -> Result {
	}
	//交易猫
	fn transfer_kitty(fromId,toId) -> Result {
	}
	//出售猫
	fn sell_kitty(kitty_id) -> Result {
	}
//算法伪代码
	//生成猫的dna
	fn generate_dna() -> Result {
		let dna_arr = vec![u8,128];
		for item in dna_arr {
			//为每个元素生成随机数
			//random_seed()方法
		}
		dna_arr
	}