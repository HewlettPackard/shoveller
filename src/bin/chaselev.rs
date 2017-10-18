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
 
/// Simple multi-threaded test for the Chase-Lev work stealing deque
/// https://github.com/aturon/crossbeam
/// http://aturon.github.io/crossbeam-doc/crossbeam/
///
/// chase_lev allows one worker that pushes to the front but N
/// "stealers" that pop from the end.

use std::thread;

extern crate crossbeam;
use crossbeam::sync::chase_lev;
use crossbeam::sync::chase_lev::{Steal};

fn main() {
    let (mut worker, stealer) = chase_lev::deque::<usize>();

    let many: usize = 12;
    let mut threads = Vec::with_capacity(12);

    let until = 1usize << 20;

    for i in 0..((many*until)>>1) {
        worker.push(i);
    }

    for _ in 0..many {
        let s = stealer.clone();
        let t = thread::spawn( move || {
            let mut count = 0;
            loop {
                if let Steal::Data(_) = s.steal() {
                    count += 1;
                } else {
                    continue;
                }
                if count >= until {
                    break;
                }
            }
        });
        threads.push(t);
    }

    for i in 0..((many*until)>>1) {
        worker.push(i);
    }

    for entry in threads {
        let _ = entry.join();
    }
}
