use hivex_rs;

fn main() {
    //
    let start = std::time::Instant::now();
    let mut hive = hivex_rs::hive::Hive::new("./test_data/SOFTWARE").unwrap();
    //
    // println!("{:?}", hive);
    while let Some(node) = hive.next() {
        // println!("{}", node.unwrap());
    }

    println!("Ms: {}", start.elapsed().as_millis());
}
