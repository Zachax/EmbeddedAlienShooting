/* Some tips.

# How to set memory address's content to zero?

core::ptr::write_volatile(ADDRESS, 0);

# How to read memory address's content?

let value = core::ptr::read_volatile(ADDRESS);
*/

use crate::{xil, CHANNEL, CONTROL, RGB, note};

use xil::{usleep};

const PAGE_SIZE: usize = 10;

pub static mut PAGE: usize = 0;

/// Table for dots.
/// Indices are page, x, y, color.
/// Initialized to zero.
static mut DOTS: [[[[u8; PAGE_SIZE]; 8]; 8]; 3] = [[[[0; PAGE_SIZE]; 8]; 8]; 3];

/// TODO: does this function have to be unsafe?
pub fn setup_led_matrix() {
    // The screen must be reset at start.
    // Tip: use the following one-liners to flip bits on or off at ADDRESS.
    // Oh yes, it's a zero-cost lambda function in an embedded application.
    /*
    mutate_ptr(ADDR, |x| x | 1);
    mutate_ptr(ADDR, |x| x ^ 1);
    */

    // TODO: Write code that sets 6-bit values in register of DM163 chip.
    // It is recommended that every bit in that register is set to 1.
    // 6-bits and 24 "bytes", so some kind of loop structure could be nice

    //reseting screen at start is a MUST to operation (Set RST-pin to 1).

    // Basic inits
    unsafe {
        core::ptr::write_volatile(CHANNEL,0);
        core::ptr::write_volatile(CONTROL,0); // Control signals all clear
        usleep(500); // Does this actually even work?
        core::ptr::write_volatile(CONTROL,0b00001); // Rst signal
        usleep(500);
        core::ptr::write_volatile(CONTROL, 0); // Back from reset
        usleep(500);
        core::ptr::write_volatile(CONTROL, 0b00001); // Rst signal final up
        mutate_ptr(CONTROL, |x| x | 5); // Activate SDA - serial input bit
    }

    
    // Init bits on and off
	for _i in 0..6 {
		for _j in 0..24 {
            unsafe {
                // Clock up and down
                mutate_ptr(CONTROL, |x| x | 4);
                mutate_ptr(CONTROL, |x| x ^ 4);
            }
		}
	}
    
    // Just make some lights to see setup is happening
    unsafe {    
        
        core::ptr::write_volatile(RGB, 0b011011); // Just a remark that something has started
        usleep(500);
        core::ptr::write_volatile(RGB, 0); // Turn off debug toy RGB LEDs

    }

    //Final thing in this function is to set SB-bit to 1 to enable transmission to 8-bit register.
    unsafe {        
        mutate_ptr(CONTROL, |x| x | 3);     
    }
}

/// Set the value of one pixel at the LED matrix.
/// Function is unsafe because it uses global memory.
/// TODO: does this function have to be unsafe?
pub unsafe fn set_pixel(x: usize, y: usize, r: u8, g: u8, b: u8) {
    // TODO: Set new pixel value.
    // Take the parameeters and put them into the DOTS array.
    
    DOTS[0][x][y][PAGE]=b;
    DOTS[1][x][y][PAGE]=g;
    DOTS[2][x][y][PAGE]=r;
}

/// Refresh new data into the LED matrix.
/// Hint: This function is supposed to send 24-bytes and parameter x is for x-coordinate.
/// TODO: does this function have to be unsafe?
pub unsafe fn run(c: usize) {
    // TODO: Write into the LED matrix driver (8-bit data).
    // Use values from DOTS array.

    // Set latch 0
    let mut bitvalue: u8= core::ptr::read_volatile(CONTROL);
    bitvalue = bitvalue & !(1u8 << 2);
    core::ptr::write_volatile(CONTROL, bitvalue);

    // Iterate the channel pixels
    for i in 0..8 { // channel level
    	for j in 0..3 { // led level
            let mut dot = DOTS[j as usize][c as usize][i as usize][PAGE as usize];
    		for k in 0..8 { // bit level
				if dot & 0b10000000 != 0 {
                    mutate_ptr(CONTROL, |x| x | 5); // SDA 1
				} else {
                    // Set SDA to 0
                    mutate_ptr(CONTROL, |x| x ^ 5); // SDA 0
				}
                mutate_ptr(CONTROL, |x| x | 4); // Clock up
                mutate_ptr(CONTROL, |x| x ^ 4); // Clock down
				dot <<= 1;
			}
		}
	}
	latch();
	// clock should go 0
    let mut clockvalue: u8 = core::ptr::read_volatile(CONTROL);
    clockvalue = clockvalue & !(1u8 << 4);
    core::ptr::write_volatile(CONTROL, clockvalue);
}

/// Latch signal for the colors shield.
/// See `colorsshield.pdf` for how latching works.
/// TODO: does this function have to be unsafe?
unsafe fn latch() {
    // Latch signal up and down
    mutate_ptr(CONTROL, |x| x | 2);
    mutate_ptr(CONTROL, |x| x ^ 2);
}

/// Set one channel as active.
/// TODO: does this function have to be unsafe?
pub unsafe fn open_line(i: u8) {
    // Tip: use a `match` statement for the parameter:
    /*
    match i {
        0 => {},
        _ => {},
    }
    */

    match i {
		0 => {
            core::ptr::write_volatile(CHANNEL, 0b00000001);
        }
		1 => {
            core::ptr::write_volatile(CHANNEL, 0b00000010);
        }
		2 => {
            core::ptr::write_volatile(CHANNEL, 0b00000100);
        }
		3 => {
            core::ptr::write_volatile(CHANNEL, 0b00001000);
        }
		4 => {
            core::ptr::write_volatile(CHANNEL, 0b00010000);
        }
		5 =>{
            core::ptr::write_volatile(CHANNEL, 0b00100000)
        }
		6 => {
            core::ptr::write_volatile(CHANNEL, 0b01000000);
        }
		7 => {
            core::ptr::write_volatile(CHANNEL, 0b10000000);
        }
		_ => {
            core::ptr::write_volatile(CHANNEL, 0);
        }
	}
}

/// A helper one-liner for mutating raw pointers at given address.
/// You shouldn't need to change this.
///
/// # How to use
///
/// Set a bit to high.
///
/// ```ignore
/// mutate_ptr(ADDR, |x| x | 1);
/// ```
///
/// Flip bit's value.
///
/// ```ignore
/// mutate_ptr(ADDR, |x| x ^ 1);
/// ```
///
/// TODO: does this function have to be unsafe?
unsafe fn mutate_ptr<A, F>(addr: *mut A, mutate_fn: F)
where
    F: FnOnce(A) -> A,
{
    let prev = core::ptr::read_volatile(addr);
    let new = mutate_fn(prev);
    core::ptr::write_volatile(addr, new);
}
