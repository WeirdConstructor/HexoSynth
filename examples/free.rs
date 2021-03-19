use hexosynth::nodes::NodeProg;

fn main() {
    loop {
        let mut vec = vec![];
        for _ in 0..10000 {
            vec.push(NodeProg::new(100, 100, 100));
        }
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}
