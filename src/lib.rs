#![cfg(test)]
#![feature(time2)]

pub mod rwcell;
pub use rwcell::RWCell;
mod test;

use std::sync::Arc;
use std::ops::{ Deref, DerefMut };
use std::convert::{ AsRef, AsMut };
use std::borrow::{ Borrow };
use std::clone::Clone;



/// The writing-half of the RWCell type. This half can only be owned by one thread, but it can be cloned to send to other threads.
pub struct CellWrite<T>(Arc<RWCell<T>>);
unsafe impl<T> Send for CellWrite<T> {}	// Note: Those types are not Sync
impl<T> CellWrite<T> {
	/// Put a new value into cell
	pub fn set(&mut self, value: T) { unsafe { self.0.write(value); } }
}

/// The reading-half of RWCell type. This half can only be owned by one thread
pub struct CellRead<T>(Arc<RWCell<T>>);
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
pub fn make_rw_cell<T>(initial: T) -> (CellWrite<T>, CellRead<T>) {
	let irc = Arc::new(RWCell::new(initial));
	(CellWrite(irc.clone()), CellRead(irc))
}
