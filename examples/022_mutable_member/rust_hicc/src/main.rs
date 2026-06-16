use mutable_member::*;

fn main() {
    let f = DataFetcher::new(100);
    // fetch() 是 const 方法（&self），但每次调用都会更新 mutable 的访问计数。
    println!("fetch={}", f.fetch());            // 101
    println!("fetch={}", f.fetch());            // 102
    println!("access_count={}", f.access_count()); // 2
    println!("--- end main ---");
}
