fn main() {
    use std::io::Read;
    let mut data: Vec<u8> = vec![0; 1];
    let mut stdin = ::std::io::stdin();
    stdin.read(&mut data[..]).unwrap();

    let mut v: Vec<u16> = Vec::new();
    for idx in 0..256 {
        v.push(idx);
    }

    if v[data[0] as usize] == 73 {
        panic!()
    }
}
