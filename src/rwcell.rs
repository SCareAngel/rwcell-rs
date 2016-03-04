///	#!
///	Single reader, single writer wait-free cell

use std::sync::atomic::{ AtomicUsize, Ordering };
use std::cell::UnsafeCell;
use std::mem;
use std::ptr;
use std::fmt::{ Debug, Formatter, Error };

use intrusive_rc::{ IntrusiveRefCounter, IRc };

/*
	writer( x ) = (x ^ 2) | 1
	reader( x ) = (x & 1) ? x + 5 : x

	! - read position
	? - write position
	X - new values
	0 - no values
	
	00 = ? - 0 - ! | writer(00) = X - ? - ! | reader(00) = ? - 0 - !
	01 = ? - X - ! | writer(01) = X - ? - ! | reader(01) = ? - ! - 0
	02 = 0 - ? - ! | writer(02) = ? - X - ! | reader(02) = 0 - ? - !
	03 = X - ? - ! | writer(03) = ? - X - ! | reader(03) = ! - ? - 0

	04 = 0 - ! - ? | writer(04) = ? - ! - X | reader(04) = 0 - ! - ?
	05 = X - ! - ? | writer(05) = ? - ! - X | reader(05) = ! - 0 - ?
	06 = ? - ! - 0 | writer(06) = X - ! - ? | reader(06) = ? - ! - 0
	07 = ? - ! - X | writer(07) = X - ! - ? | reader(07) = ? - 0 - !

	08 = ! - ? - 0 | writer(08) = ! - X - ? | reader(08) = ! - ? - 0
	09 = ! - ? - X | writer(09) = ! - X - ? | reader(09) = 0 - ? - !
	10 = ! - 0 - ? | writer(10) = ! - ? - X | reader(10) = ! - 0 - ?
	11 = ! - X - ? | writer(11) = ! - ? - X | reader(11) = 0 - ! - ?
*/

pub struct RWCell<T> {
	marker: AtomicUsize,
	values: UnsafeCell<[T;3]>,
}

impl<T> RWCell<T> {
	pub fn new(values: T) -> IRc<RWCell<T>> {
		let new = RWCell {
			values: unsafe { UnsafeCell::new([values, mem::uninitialized(), mem::uninitialized()]) },
			marker: AtomicUsize::new(0)
		};
		IRc::new(new)
	}

	unsafe fn dispose(self) {
		let marker = self.marker.load(Ordering::Relaxed);
		let pos = Self::reader_pos(marker);
		mem::drop(ptr::read(self.inner_get(pos)));
		mem::forget(self); // do not attempt to drop anything else
	}

	pub unsafe fn read(&self) -> &mut T {
		let marker0 = self.marker.load(Ordering::Acquire);
		// calculate read position
		let pos = Self::reader_pos(marker0);

		if (marker0 & 1) == 0 {
			self.inner_get(pos)
		} else {
			// drop old values
			// mem::drop just to emphasis drop operation
			//--mem::drop(ptr::read(self.inner_get(pos)));
			let removed = ptr::read(self.inner_get(pos));

			// switch reader position to position of new values
			let marker1 = self.reader_switch(marker0);

			// calculate new read position			
			let pos = Self::reader_pos(marker1);			
			mem::drop(removed);
			self.inner_get(pos)
		}
	}

	pub unsafe fn write(&self, values: T) {
		let marker0 = self.marker.load(Ordering::Acquire);
		// calculate write position
		let pos = Self::writer_pos(marker0);

		// place values at writer position
		// writer position was empty
		ptr::write(self.inner_get(pos), values);

		let marker1 = self.marker.fetch_xor(2, Ordering::Release) ^ 2;
		if (marker1 & 1) == 0 {
			self.marker.fetch_or(1, Ordering::Relaxed);
		}
		else {
			// new values wasn't read by reader, drop it
			let pos = Self::writer_pos(marker1);

			// mem::drop just to emphasis drop operation
			//--mem::drop(ptr::read(self.inner_get(pos)));
			let removed = ptr::read(self.inner_get(pos));
			mem::drop(removed);
		}		
	}

	unsafe fn inner_get<'a>(&'a self, index: usize) -> &'a mut T { &mut (*self.values.get())[index] }

	// 4 bits for rw part
	fn rw_part(marker: usize) -> usize { marker & 15 }
	fn new_value_exists(marker: usize) -> bool { 0 != (marker & 1) }
	fn reader_pos(marker: usize) -> usize { (Self::rw_part(marker) / 4) % 3 }
	fn reader_switch(&self, marker: usize) -> usize {
		if Self::rw_part(marker) > 7 {
			self.marker.fetch_sub(7, Ordering::AcqRel) - 7
		} else {
			self.marker.fetch_add(5, Ordering::AcqRel) + 5
		}
	}

	fn writer_pos(marker: usize) -> usize { 2 - ((Self::rw_part(marker) / 2) % 3) }
}

impl<T> IntrusiveRefCounter for RWCell<T> {
	fn acquire_ref(p: ptr::Shared<Self>) {
		// sizeof(Usize) * 8 - 4 bits for ref counting
		unsafe {p.as_ref().unwrap().marker.fetch_add(16, Ordering::Relaxed);}
	}

	fn release_ref(p: ptr::Shared<Self>) {
		// sizeof(Usize) * 8 - 4 bits for ref counting
		if 16 == unsafe { p.as_ref().unwrap().marker.fetch_sub(16, Ordering::AcqRel) } {
			unsafe { Box::from_raw(p.as_mut().unwrap()).dispose() }
		}
	}	
}

unsafe impl<T: Send + Copy> Send for RWCell<T> {}


impl<T: Debug> Debug for RWCell<T> {
	fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
		let marker = self.marker.load(Ordering::Relaxed);
		write!(f, "values: {:?}, marker: {:?}", unsafe { &*self.values.get() }, marker)
	}
}