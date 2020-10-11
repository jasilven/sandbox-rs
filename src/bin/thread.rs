use std::thread;

fn main() {
    println!("main thread id: {:?} ", thread::current().id());

    let t1 = thread::spawn(|| {
        println!("t1 id: {:?}", thread::current().id());

        100
    });

    let t2 = thread::spawn(|| {
        println!("t2 id: {:?}", thread::current().id());
        200
    });

    let res_1 = t1.join().expect("thread 1 panicked");
    let res_2 = t2.join().expect("thread 2 panicked");

    println!("res1: {}", res_1);
    println!("res2: {}", res_2);
}
