//尝试定义链表结构，解决复杂度问题
use List::*;
use crate::substratekitties::Kitty;
pub enum List<T: system::Trait>{
    Cons(((T::AccountId, u32),u32),Box<List<T>>),
    Nil
}
impl<T: system::Trait> List<T>{
    fn new() -> List<T>{
        Nil
    }
    fn prepend(self, elem: ((T::AccountId, u32),u32)) -> List<T> {
        // `Cons` 同样为 List 类型
        Cons(elem, Box::new(self))
    }

    fn len(&self) -> u32 {
        match *self {
            Cons(_, ref tail) => 1 + tail.len(),
            Nil => 0
        }
    }

    fn remove(self,elem: ((T::AccountId, u32),u32)) -> List<T>{
        //链表删除操作
        Nil
    }
}