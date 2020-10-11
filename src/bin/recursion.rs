fn recur(i: usize, limit: usize) -> usize {
    if i == limit {
        return i;
    } else {
        1 + recur(1 + i, limit)
    }
}

fn main() {
    for limit in 100000.. {
        println!("limit {}", limit);
        recur(0, limit);
    }
}
