use bitcoin::consensus::Decodable;
use ord::SatPoint;
use rocksdb::IteratorMode;
use std::env::current_dir;
use std::io;

fn main() {
  let current_dir = current_dir().unwrap();
  println!("{:?}", current_dir);
  let rdb = rocksdb::DB::open_default(current_dir.join("_cache").join("rocksdb")).unwrap();
  let iter = rdb.iterator(IteratorMode::Start); // Always iterates forward
  for item in iter.take(3) {
    let (key, _value) = item.unwrap();
    let mut a: [u8; 8] = [0; 8];
    a.copy_from_slice(key.as_ref());
    println!("Saw {:?}, {}", key, u64::from_le_bytes(a));
  }
  let a = rdb.get(0u64.to_le_bytes()).unwrap();
  let b = SatPoint::consensus_decode(&mut io::Cursor::new(a.unwrap())).unwrap();
  println!("{:?}", b);
}
