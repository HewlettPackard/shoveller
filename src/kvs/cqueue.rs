/*
 * Shoveller - scalable, memory-capacity efficient key-value store for very large scale machines
 *
 * (c) 2017 Hewlett Packard Enterprise Development LP.
 *
 * This program is free software: you can redistribute it and/or modify it under the terms of the 
 * GNU Lesser General Public License as published by the Free Software Foundation, either version 3 
 * of the License, or (at your option) any later version. This program is distributed in the hope that 
 * it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or 
 * FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License along with this program.
 * If not, see <http://www.gnu.org/licenses/>. As an exception, the copyright holders of this Library 
 * grant you permission to (i) compile an Application with the Library, and (ii) distribute the Application 
 * containing code generated by the Library and added to the Application during this compilation process 
 * under terms of your choice, provided you also meet the terms and conditions of the Application license.
 */

/*
 * A concurrent queue for exactly two threads.
 * One may push, the other may pop, but no locks or atomics are used.
 * The size of the queue remains fixed after compilation.
 *
 * XXX AS-YET UNTESTED IMPLEMENTATION
 */

#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

use std::ptr;

/// Number of entries in the queue.
/// Must be a power of two.
const CQEUEUE_ENTRIES: usize = 2048;

pub struct CQueue {
	head: usize,
	/// The objects within the queue are u64. Keep it sandwiched
	/// between head and tail so they exist in their own cache lines.
	array: [u64; CQEUEUE_ENTRIES],
	tail: usize,
}

macro_rules! cqnext {
    ( $idx:expr ) => (
    	($idx + 1) & (CQEUEUE_ENTRIES - 1)
    )
}

impl CQueue {
	
	pub fn new() -> Self {
		CQueue {
			head: 1,
			array: [0u64; 2048],
			tail: 0,
		}
	}

	/// Read both head and tail as volatile to check if empty
	#[inline(always)]
	pub fn empty(&self) -> bool {
		let t: usize = unsafe {
			ptr::read_volatile(&self.tail)
		};
		let h: usize = unsafe {
			ptr::read_volatile(&self.head)
		};
		cqnext!(t) == h
	}

	/// Read only tail as volatile to check if empty.
	/// Used by the producer (pushes).
	#[inline(always)]
	pub fn empty_tail(&self) -> bool {
		let t: usize = unsafe {
			ptr::read_volatile(&self.tail)
		};
		cqnext!(t) == self.head
	}

	/// Read only head as volatile to check if empty.
	/// Used by the consumer (pops).
	#[inline(always)]
	pub fn empty_head(&self) -> bool {
		let h: usize = unsafe {
			ptr::read_volatile(&self.head)
		};
		cqnext!(self.tail) == h
	}

	#[inline(always)]
	pub fn full_tail(&self) -> bool {
		let t: usize = unsafe {
			ptr::read_volatile(&self.tail)
		};
		self.head == t
	}

	#[cfg(debug_assertions)]
	#[inline(always)]
	fn __pop(&self, val: &mut u64) {
		*val = self.array[self.tail];
		unsafe {
			let t: *mut usize = &self.tail as *const _ as *mut _;
			ptr::write_volatile(t, cqnext!(self.tail));
		}
	}

	#[cfg(not(debug_assertions))]
	#[inline(always)]
	fn __pop(&self, val: &mut u64) {
		*val = unsafe {
			*self.array.get_unchecked(self.tail as usize)
		};
		unsafe {
			let t: *mut usize = &self.tail as *const _ as *mut _;
			ptr::write_volatile(t, cqnext!(self.tail));
		}
	}

	#[inline(always)]
	pub fn pop_try(&self, val: &mut u64) -> bool {
		if self.empty_head() {
			false
		} else {
			self.__pop(val);
			true
		}
	}

	#[cfg(debug_assertions)]
	#[inline(always)]
	fn __push(&self, val: u64) {
		let prior: usize = if self.head > 0 {
			self.head - 1
		} else {
			CQEUEUE_ENTRIES - 1
		};
		unsafe {
			let v = &self.array[prior] as *const _ as *mut _;
			ptr::write(v, val);
			let h: *mut usize = &self.head as *const _ as *mut _;
			ptr::write_volatile(h, cqnext!(self.head));
		}
	}

	#[cfg(not(debug_assertions))]
	#[inline(always)]
	fn __push(&self, val: u64) {
		let prior: usize = if self.head > 0 {
			self.head - 1
		} else {
			CQEUEUE_ENTRIES - 1
		};
		unsafe {
			*(self.array.get_unchecked(prior)
				as *const _ as *mut _) = val;
			let h: *mut usize = &self.head as *const _ as *mut _;
			ptr::write_volatile(h, cqnext!(self.head));
		}
	}

	#[inline(always)]
	pub fn push_try(&self, val: u64) -> bool {
		if self.full_tail() {
			false
		} else {
			self.__push(val);
			true
		}
	}
}
