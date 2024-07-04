use thread_pool::ThreadPool;

fn main() {
    let mut tp = ThreadPool::new(4).unwrap();

    for i in 0..30 {
        tp.spawn(move || {
            println!("Hola deade el thread ejecutado en {i}");
        });
    }

    println!("{tp}");
}
