#![no_implicit_prelude]
use ::bitset::BitFlag;

#[derive(BitFlag, Clone, Copy, Debug)]
enum MyFlag {
    Foo,
    Bar,
    Bazz = 3,
    Bla,
}

fn main() {
    use MyFlag::*;

    use ::core::iter::Extend;
    use ::std::println;

    let mut set = Foo | Bar;
    println!("set is {:?}", set);
    println!("{:?}", set.pop());
    println!("{} {}", set & Foo, set & Bar);
    println!("set is {:?}, bits={:b}", set, set.bits());
    set.extend([Foo, Bar, Bazz, Bla]);
    println!("set is {:?}, bits={:b}", set, set.bits());
}
