use std::thread::{self, sleep};
use std::time::Duration;

fn main() {
    println!("So, we start the program here!");
    let t1 = thread::spawn( move || {
        sleep(Duration::from_millis(200));
        println!("The long running tasks finish last!");
    });
    let t2 = thread::spawn(move || {
        sleep(Duration::from_millis(100));
        println!("we can chain callback..");
        let t3 = thread::spawn( move || {
            sleep(Duration::from_millis(50));
            println!("..like this");
        });
        t3.join().unwrap();
    });
    println!("The tasks run concurrently!");
    t1.join().unwrap();
    t2.join().unwrap();
}
