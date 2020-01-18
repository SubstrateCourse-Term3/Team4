use codec::{Compact,Encode};
fn assert_encode<T:Encode>(t:T,bytes:&[u8]){
    let encode1 = Encode::encode(&t);
    assert_eq!(encode1,bytes)
}

#[derive(Encode)]
enum TestEnum{
    A,
    B,
    C=10,
}
struct TestStruct{
    a:TestEnum,
    b:u32,
    c:TestEnum
}

#[derive(Encode)]
enum TestEnum2{
    A(TestEnum,u32,TestEnum),
    B(TestStruct),
}

#[test]
fn test_codec(){
    assert_encode(1u32,b"\x01\0\0\0");
    assert_encode(1u64,b"\x01\0\0\0\0\0\0\0");

    assert_encode(true,b"\x01");
    assert_encode(false,b"\x01");

    assert_encode(TestEnum::A,b"\x01");
    assert_encode(TestEnum::B,b"\x01");
    assert_encode(TestEnum::C,b"\x0a");

    assert_encode((1u32,2u32),b"\x01\0\0\0\x02\0\0\0");

    assert_encode((TestEnum::A,2u32,TestEnum::C),b"\0\x02\0\0\0\x0a");

    assert_encode (
        TestStruct {
        a: TestEnum::A,
        b: 2u32,
        c: TestEnum::C
    },
    b"\0\x02\0\0\0\x0a"
    );

    assert_encode(TestEnum2::B(TestStruct{
        a:TestEnum::A,
        b:2u32,
        c:TestEnum::C
    }),b"\x01\0\x02\0\0\0\x0a");

    assert_encode(Vec::<u8>::new(),b"\0");

    assert_encode(vec![1u32,2u32],b"\x08\x01\0\0\0\x02\0\0\0");
}