use std::ptr;
use std::ops::Deref;

/// Trait allow type to use owned reference counter via IRc
pub trait IntrusiveRefCounter {
	fn acquire_ref(p: ptr::Shared<Self>);
	fn release_ref(p: ptr::Shared<Self>);
}

/// Pointer wrapper. Use IntrusiveRefCounter to track reference count
pub struct IRc<T: IntrusiveRefCounter>(ptr::Shared<T>);

impl<T: IntrusiveRefCounter> IRc<T> {
	/// Create new IRc object
	pub fn new(value: T) -> IRc<T> {
		let boxed = Box::new(value);
		let raw = Box::into_raw(boxed);
		let shared = unsafe { ptr::Shared::new(raw) };
		T::acquire_ref(shared);
		IRc(shared)
	}
}

impl<T: IntrusiveRefCounter> Drop for IRc<T> {
	/// Release one reference
	fn drop(&mut self) {
		T::release_ref(self.0);
	}
}

impl<T: IntrusiveRefCounter> Deref for IRc<T> {
	type Target = T;
	fn deref(&self) -> &T { unsafe { self.0.as_ref().unwrap() } }
}

impl<T: IntrusiveRefCounter> Clone for IRc<T> {
	/// Create a copy of pointer
	fn clone(&self) -> IRc<T> {
		T::acquire_ref(self.0);
		IRc(self.0)
	}
}