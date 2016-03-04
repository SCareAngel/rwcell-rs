#![feature(shared, ptr_as_ref, time2)]


pub mod rwcell;
pub mod intrusive_rc;	// TODO: Move to external crate

use intrusive_rc::IRc;
use std::ops::{ Deref, DerefMut };
use std::convert::{ AsRef, AsMut };
use std::borrow::{ Borrow };
use std::clone::Clone;



/// The writing-half of the RWCell type. This half can only be owned by one thread, but it can be cloned to send to other threads.
pub struct CellWrite<T>(IRc<rwcell::RWCell<T>>);
unsafe impl<T> Send for CellWrite<T> {}	// Note: Those types are not Sync
impl<T> CellWrite<T> {
	/// Put a new value into cell
	fn set(&mut self, value: T) { unsafe { self.0.write(value); } }
}

/// The reading-half of RWCell type. This half can only be owned by one thread
pub struct CellRead<T>(IRc<rwcell::RWCell<T>>);
unsafe impl<T> Send for CellRead<T> {}	// Note: Those types are not Sync
impl<T> Deref for CellRead<T> {
	type Target = T;
	fn deref(&self) -> &T { self.as_ref() }
}
impl<T> DerefMut for CellRead<T> { fn deref_mut(&mut self) -> &mut T { self.as_mut() } }
impl<T> AsRef<T> for CellRead<T> { fn as_ref(&self) -> &T { unsafe { self.0.read() } } }
impl<T> AsMut<T> for CellRead<T> { fn as_mut(&mut self) -> &mut T { unsafe { self.0.read() } } }
impl<T> Borrow<T> for CellRead<T> { fn borrow(&self) -> &T { self.as_ref() } }

/// Creates a new RWCell, returning the writing/reading halves.
fn make_rw_cell<T>(initial: T) -> (CellWrite<T>, CellRead<T>) {
	let irc = rwcell::RWCell::new(initial);
	(CellWrite(irc.clone()), CellRead(irc))
}

// #[cfg(test)]
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

	match rth.join() {
		Ok(()) => {}
		Err(ref boxed) => {}
	}

	println!("rwcell\t- finish in: {:?}", time.elapsed().unwrap());
}

#[test]
fn smoke_test2() {
	let time = SystemTime::now();
	let (mut w, mut r) = mpsc::channel();

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
		w.send(i);
		sleep(Duration::new(0, 10));
	}

	drop(w);

	match rth.join() {
		Ok(()) => {}
		Err(ref boxed) => {}
	}

	println!("mpsc\t- finish in: {:?}", time.elapsed().unwrap());
}


}