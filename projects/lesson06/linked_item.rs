use support::{StorageMap, Parameter};
use sp_runtime::traits::Member;
use codec::{Encode, Decode};

#[cfg_attr(feature = "std", derive(Debug, PartialEq, Eq))]
#[derive(Encode, Decode)]
pub struct LinkedItem<Value>{
    pub prev: Option<Value>,
    pub next: Option<Value>,
}
pub struct LinkedList<Storage, Key, Value>(rstd::marker::PhantomData<(Storage, Key, Value)>);
impl<Storage, Key, Value> LinkedList<Storage, Key, Value> where 
    Value: Parameter + Member + Copy,
    Key: Parameter,
    Storage: StorageMap<(Key, Option<Value>), LinkedItem<Value>, Query = Option<LinkedItem<Value>>>
{
    fn read_head(key: &Key) -> LinkedItem<Value>{
        Self::read(key,None)
    }
    fn write_head(key: &Key, item: LinkedItem<Value>){
        Self::write(key, None, item)
    }
    pub fn read(account: &Key, key: Option<Value>) -> LinkedItem<Value>{
        Storage::get(&(account.clone(), key)).unwrap_or_else(|| LinkedItem{
            prev:None,
            next:None,
        })
    }
    fn write(account: &Key, key: Option<Value>, item: LinkedItem<Value>){
        Storage::insert(&(account.clone(), key), item);
    }

///作业:完成append和remove函数
    pub fn append(account: &Key, kitty_id: Value){
        let head  = Self::read_head(account);
        let new_head = LinkedItem{
            prev: Some(kitty_id),
            next: head.next
        };
        Self::write_head(account, new_head);

        let prev = Self::read(account, head.prev);
        let new_prev = LinkedItem{
            prev: prev.prev,
            next: Some(kitty_id)
        };
        Self::write(account, head.prev, new_prev);

        let item = LinkedItem{
            prev: head.prev,
            next: None,
        };
        Self::write(account, Some(kitty_id), item);
    }
    pub fn remove(account: &Key, kitty_id: Value){

        if let Some(item) = Storage::take(&(account.clone(), Some(kitty_id))){
            let prev = Self::read(account, item.prev);
            let new_prev = LinkedItem{
                prev: prev.prev,
                next: item.next,
            };
            Self::write(account, item.prev, new_prev);

            let next = Self::read(account, item.next);
            let new_next = LinkedItem {
                prev: item.prev,
                next: next.next
            };
            Self::write(account, item.next, new_next);
        }
    }
}