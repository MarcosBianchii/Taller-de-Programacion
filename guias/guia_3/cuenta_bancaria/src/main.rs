use std::sync::{Arc, Mutex};
use std::thread;

struct Account(i32);

impl Account {
    fn deposit(&mut self, amount: i32) {
        println!("op: deposit {}, available funds: {:?}", amount, self.0);
        self.0 += amount;
    }

    fn withdraw(&mut self, amount: i32) {
        println!("op: withdraw {}, available funds: {}", amount, self.0);
        if self.0 >= amount {
            self.0 -= amount;
        } else {
            panic!("Error: Insufficient funds.")
        }
    }

    fn balance(&self) -> i32 {
        self.0
    }
}

fn main() {
    let account = Arc::new(Mutex::new(Account(0)));

    let send = Arc::clone(&account);
    let customer1_handle = thread::spawn(move || {
        send.lock().unwrap().deposit(40);
    });

    let send = Arc::clone(&account);
    let customer2_handle = thread::spawn(move || loop {
        if let Ok(mut guard) = send.try_lock() {
            if 30 <= guard.balance() {
                guard.withdraw(30);
                break;
            }
        }
    });

    let send = Arc::clone(&account);
    let customer3_handle = thread::spawn(move || {
        send.lock().unwrap().deposit(60);
    });

    let send = Arc::clone(&account);
    let customer4_handle = thread::spawn(move || loop {
        if let Ok(mut guard) = send.try_lock() {
            if 70 <= guard.balance() {
                guard.withdraw(70);
                break;
            }
        }
    });

    let handles = vec![
        customer1_handle,
        customer2_handle,
        customer3_handle,
        customer4_handle,
    ];

    for handle in handles {
        handle.join().unwrap();
    }

    let savings = account.lock().unwrap().balance();

    println!("Balance: {:?}", savings);
}
