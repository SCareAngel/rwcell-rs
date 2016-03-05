#[cfg(test)]
mod tests {

use std::thread::{ spawn, sleep };
use std::time::Duration;
use std::sync::mpsc;
use std::time::SystemTime;


#[test]
fn smoke_test() {
	let time = SystemTime::now();
	let (mut w, r) = ::make_rw_cell(3);

	let rth = spawn(move || {
		println!("reader ready");
		loop {
			let value =  *r;
			if value == 9999 { break; }

			if value < 3 {
				println!("Bad value {}", value);
			}
			sleep(Duration::new(0, 10));
		}
	});

	for i in 4..10000 {
		w.set(i);
		sleep(Duration::new(0, 10));
	}

	rth.join().unwrap();

	println!("rwcell\t- finish in: {:?}", time.elapsed().unwrap());
}

#[test]
fn smoke_test2() {
	let time = SystemTime::now();
	let (w, r) = mpsc::channel();

	let rth = spawn(move || {
		println!("reader ready");
		loop {
			let value =  r.recv();
			if value.is_err() { break; }
			let value = value.unwrap();

			if value < 3 {
				println!("Bad value {}", value);
			}			
			sleep(Duration::new(0, 10));
		}
	});

	for i in 4..10000 {
		w.send(i).unwrap();
		sleep(Duration::new(0, 10));
	}

	drop(w);

	rth.join().unwrap();

	println!("mpsc\t- finish in: {:?}", time.elapsed().unwrap());
}


}