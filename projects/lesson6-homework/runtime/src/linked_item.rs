//通用引入
use support::{StorageMap, Parameter};
use sp_runtime::traits::Member;
use codec::{Encode, Decode};

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct LinkedItem<Value> {
	pub prev: Option<Value>,
	pub next: Option<Value>,
}

//数据结构 范型参数 PhantomData不会产生实际大小类型避免错误
pub struct LinkedList<Storage, Key, Value>(rstd::marker::PhantomData<(Storage, Key, Value)>);

impl<Storage, Key, Value> LinkedList<Storage, Key, Value> where
    Value: Parameter + Member + Copy,
    Key: Parameter,
    Storage: StorageMap<(Key, Option<Value>), LinkedItem<Value>, Query = Option<LinkedItem<Value>>>,
{
    fn read_head(key: &Key) -> LinkedItem<Value> {
 		Self::read(key, None)
 	}

  	fn write_head(account: &Key, item: LinkedItem<Value>) {
 		Self::write(account, None, item);
 	}

  	fn read(key: &Key, value: Option<Value>) -> LinkedItem<Value> {
 		Storage::get((&key, value)).unwrap_or_else(|| LinkedItem {
 			prev: None,
 			next: None,
 		})
 	}

  	fn write(key: &Key, value: Option<Value>, item: LinkedItem<Value>) {
 		Storage::insert((&key, value), item);
 	}

    pub fn append(key: &Key, value: Value) {
        // 作业：实现 append
		//找出要插入的位置以及该位置的元素 改表头
		let head=Self::read_head(key);
		let new_head=LinkedItem{
		prev:Some(value),
		next:head.next
		};

		Self::write_head(key,new_head);

		//旧链链尾  改表尾
		let prev=Self::read(key,head.prev);
		//处理错误
		/**
fn main() {
    let mut count: u32 = 1;
    let mut num: u64 = 1;
    let mut primes: Vec<u64> = Vec::new();
    primes.push(2);

    while count < 10001 {
        num += 2;
        if vector_is_prime(num, &primes) {
            count += 1;
            primes.push(num);
        }
    }
}

fn vector_is_prime(num: u64, p: &[u64]) -> bool {
    for &i in p {
        if num > i && num % i != 0 {
            return false;
        }
    }
    true
}
*/
		let new_prev=LinkedItem{prev:prev.prev,next:Some(&value),};

		Self::write(key,head.prev,new_prev);

		let item=LinkedItem{
			prev:head.prev,
			next:None
		};
		Self::write(key,Some(&value),item);
    }




    pub fn remove(key: &Key, value: Value) {
        // 作业：实现 remove
		if let Some (item)=Storage::take(&(key.clone()),Some(value)){
			//获取当前位置 改表头
			let prev=Self::read(key,item.prev);
			let new_prev=LinkedItem{
				prev:prev.prev,
				next:item.next
			};

			Self::write(key,item.prev,new_prev);

			//改表尾
			let next=Self::read(key,item.next);
			let new_next=LinkedItem{
				prev:item.prev,
				next:next.next,
			};
			Self::write(key,item.next,new_next);
			Storage::remove((key,Some(value)));
		}
    }
}