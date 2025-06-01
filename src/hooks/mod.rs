pub fn run() {
    println!("Starting hook thread.");
    for i in 0..5 {
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("Hello from hook thread! {i}");
    }
}
