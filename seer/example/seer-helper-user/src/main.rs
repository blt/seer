#[macro_use]
extern crate seer_helper;
seer_helper_init!();

#[derive(Debug)]
struct Test {
    a: u32,
    b: u64,
}

fn main() {
    let mut t = Test {a:0,b:0};
    seer_helper::mksym(&mut t);
    if t.a == 123 && t.b == 321 {panic!()}
}
