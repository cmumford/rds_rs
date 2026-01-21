// or with custom logic (like a generator)
fn fibonacci_up_to(max: u64) -> impl Iterator<Item = u64> {
    let mut a = 0;
    let mut b = 1;
    std::iter::from_fn(move || {
        let next = a + b;
        if next > max {
            return None;
        }
        let result = a;
        a = b;
        b = next;
        Some(result)
    })
}

fn main() {
    for num in fibonacci_up_to(100) {
        println!("{}", num);
    }
}
