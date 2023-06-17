use wnd_macros::*;

#[test]
#[should_panic]
#[todo_attr("not yet done")]
fn todo_fn() {
    
}

#[thread]
fn thread_fn(n: u64, tx: std::sync::mpsc::Sender<u64>) {
    std::thread::sleep(
        std::time::Duration::from_millis(n)
    );
    tx.send(n).unwrap();
}

#[test]
fn thread_functions() {
    let (tx, rx) = std::sync::mpsc::channel();
    thread_fn(5, tx.clone());
    thread_fn(6, tx.clone());
    thread_fn(7, tx.clone());
    thread_fn(8, tx.clone());
    thread_fn(9, tx.clone());
    let mut b = [0u64; 5];
    for n in 0..5 {
        b[n] = rx.recv().unwrap();
    }
    assert_eq!(b, [5, 6, 7, 8, 9])
}