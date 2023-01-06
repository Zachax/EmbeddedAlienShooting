//! Implements the LED-blinker course work on Xilinx PYNQ-Z1 SoC.
//!
//! Uses functionality defined in Xilinx's board support package (BSP) via Rust's foreign function interface (FFI).

// Do not include Rust standard library.
// Rust standard library is not available for bare metal Cortex-A9.
// Thus we use [core](https://doc.rust-lang.org/core/)-library.
#![no_std]
// Open feature gates to certain, currently WIP, features which might become part of Rust in future.
#![feature(start)]

// Define crate's module hierarchy.
mod interrupt;
mod pixel;
mod print;

// `xil_sys` contains the Xilinx Cortex-A9 board support package (BSP) and a Rust FFI.
// We rename the module here as `xil`.
use xil_sys as xil;

// Re-import symbols from pixel without the `pixel::` prefix.
use pixel::*;

// Rust `core` imports for using C-style void-pointers and info for a custom panic implementation.
use core::{ffi::c_void, panic::PanicInfo};

// Frequency change is used for game difficulty adjusting
use crate::interrupt::change_freq;

// Declare static globals like in the C-version.
// This is a reasonable way of communicating between threads in interrupt-driven concurrency.
pub static mut A_GLOBAL: usize = 0;

// Define the address of the ordinary LED interface in physical memory.
// Putting bits into the LED address the right way may cause desired blinking of hardware LEDs.
// FIXME: 0x00000000 is not the LED address.
// The correct address can be found in some of the provided documentation.
pub const LED_ADDRESS: *mut u8 = 0x41200000 as *mut u8;

// Define other addresses
pub const CHANNEL: *mut u8 = 0x41220000 as *mut u8;
pub const CONTROL: *mut u8 = 0x41220008 as *mut u8;
pub const INPUTS: *mut u32 = 0xE000A068 as *mut u32;
pub const RGB: *mut u8 = 0x41240000 as *mut u8;

// Game variables
pub static mut OPEN_CHANNEL: usize = 0;
pub static mut ALIEN_X: usize = 0;
pub static mut ALIEN_Y: usize = 0;
pub static mut SHIP_X: usize = 3;
pub static mut SHIP_Y: usize = 7;
pub static mut BULLET_X: usize = 0;
pub static mut BULLET_Y: usize = 0;
pub static mut INCREMENT: i8 = 1;
pub static mut FREQUENCY: u16 = 10;
pub static mut SCORE: u16 = 0;
pub static mut SHIELD: u16 = 5;
pub static mut IS_WON: u16 = 0; // Flag for game won
pub static mut IS_LOST: u16 = 0; // flag for game lose
pub static BULLET_HAVEN: usize = 50; // "hiding place" for bullet when off board

// Assembly stuff for blinking leds
use core::arch::global_asm;
//global_asm!(include_str!("blinker.S"));

global_asm!("
.data
	dir: .byte 1		//8-bit variable for direction

.text

.global blinker

blinker:
	ldr r0, =0x41200000 // Load LED address value
	ldrb r1, [r0] // Read previous led value
	cmp r1,#0 // Init for first round
	bne directioncheck
	add r1,#1
	b storing
directioncheck:
	ldr r2, =dir // Read direction variable address
	ldr r3, [r2] // Get address value
	cmp r3,#1
	beq leftshift
	b rightshift
leftshift:
	mov r1,r1, lsl#1 // Bit shift left
	cmp r1,#7 // Check if gone over to left
	bgt flipdirectionright
	b storing
rightshift:
	mov r1,r1, lsr#1 // Bit shift right
	cmp r1,#1 // Check if gone over to right
	ble flipdirectionleft
	b storing
flipdirectionright:
	mov r3,#0
	b storing
flipdirectionleft:
	mov r3,#1
storing:
	str r3, [r2] // Store direction used
	str r1, [r0] // Store new led value
	bx lr					//Return to place where this function has been called.
");

extern "C" {
    fn blinker() -> u32;
}

// The #[start] attribute tell's the cross-compiler where to start executing.
// Normally it is not needed.
// UnderSCORE before the argument signals that the parameter is not used.
#[start]
fn main(_argc: isize, _argv: *const *const u8) -> isize {
    // Initialize board interrupt functions.
    // N.B. Do not touch this function, concurrency is set up here.
    interrupt::init();
    
    note("Game init started.");

    // Init game variables
    unsafe {
        init_game();
    }
    
    setup_led_matrix();

    // An unsafe block for setting up the LED-matrix using the C-API, and for touching a static global.
    unsafe {
        // Setting a static global variable requires an `unsafe` block in Rust.
        // Compiler can not verify soundness in a case where an interrupt causes simultaneous access from another thread.
        // Now it is our responsibility to make sure that this does not happen.
        A_GLOBAL = 0;
    }

    unsafe {
        // Enable interrupts.
        // Now control flow can change from main loop to an interrupt handler.
        // This change of control flow happens when an interrupt request (IRQ) is asserted.
        // Direct calls to C API functions (`xil::*`) are `unsafe` by default.
        // The compiler does not verify soundness of C code.
        xil::Xil_ExceptionEnable();
    }

    // Prints up to 64 characters using standard Rust [print formatting](https://doc.rust-lang.org/std/fmt/index.html).
    // You should see this using PuTTY.
    note("Rust application initialized!");

    // Empty loop to keep the program running while the interrupt handlers do all the work.
    loop {}
}

/// Interrupt handler for switches and buttons.
///
/// Pressing a button or switching a switch causes an GPIO interrupt.
/// This function is used to handle that interrupt.
///
/// Connected buttons are at bank 2.
/// This is defined during hardware synthesis using Vivado (e.g. could also be in bank 3).
///
/// # Arguments
///
/// * `status` - a binding containing one flipped bit to match the source of the
///   interrupt. See line comments of contained `match` statement.
pub unsafe extern "C" fn button_handler(callback_ref: *mut c_void, _bank: u32, status: u32) {
    // Don't mind me, line is for brevity
    // N.B. Removing this line is totally okay
    let _gpio = callback_ref as *mut xil::XGpioPs;

    // TODO: Write code here
    // Tip: use a match-statement to pattern-match button status. The match
    // statement takes the `status` parameter binding and matches it to
    // different binary patterns (eg. 0b001 for decimal 1, or 0b100 for
    // decimal 4). You can use binary, decimal or hex for the match, but I
    // found the binary representation more readable.
    
    // Reset ship graphics
    set_pixel(SHIP_X, SHIP_Y,0,0,0); // center
    set_pixel(SHIP_X-1, SHIP_Y,0,0,0); // left
    set_pixel(SHIP_X+1, SHIP_Y,0,0,0); // right
    set_pixel(SHIP_X, SHIP_Y-1,0,0,0); // top

    //Hint: Status==0x01 ->btn0, Status==0x02->btn1, Status==0x04->btn2, Status==0x08-> btn3, Status==0x10->SW0, Status==0x20 -> SW1
    match status {
        // No buttons are pressed
        0b000000 => {},
        // TODO: match into a pattern here
        //If true, btn0 was used to trigger interrupt
        0x01 => {
            // Move ship right
            SHIP_X += 1;
        }
        0x02 => {

        }
        0x04 => {
            // Shoot
            if BULLET_Y > 10 { // number rolls over 0 to max
                BULLET_X = SHIP_X;
                BULLET_Y = 6;
            }
        }
        0x08 => {
            // Move ship left
            SHIP_X -= 1;
        }
        0x10 => {
            // Restart game
            init_game(); 
        }
        0x20 => {
            // Swap game speed -> easy/hard difficulty level    
            if FREQUENCY == 10 {
                FREQUENCY = 20;
            } else {
                FREQUENCY = 10;
            }
            change_freq(FREQUENCY.into());
            note("Changed game speed.");
        }
        // `_` is the 'rest' pattern, that is handled if no other variant matches above
        _ => {},
    }


    // Prevent ship from moving over edges to right
    if SHIP_X > 6 {
        SHIP_X = 6;
    }

    // Prevent ship from moving over edges to left
    if SHIP_X < 1 {
        SHIP_X = 1;
    }

    // End of your code
}

/// Timer interrupt handler for led matrix update.
/// The function updates only one line (`CHANNEL`) of the matrix per call, but sets `channel` as the next line to be updated.
/// `pub extern "C"` qualifier is required to allow passing the handler to the C API.
pub unsafe extern "C" fn tick_handler(callback_ref: *mut c_void) {
    // Exceptions need to be disabled during screen update.
    xil::Xil_ExceptionDisable();

    // TODO: Write code here
    if OPEN_CHANNEL > 7 {
		OPEN_CHANNEL = 0;
	}
	open_line(99);
	run(OPEN_CHANNEL);
	open_line(OPEN_CHANNEL.try_into().unwrap());
	OPEN_CHANNEL+=1;
    // End of your code

    // Cast `void*` received from the C API to the "Triple Timer Counter" (TTC) instance pointer.
    // The C API needs to use void pointers to pass data around.
    // The C specification does not describe abstract data types (ADT).
    let ttc = callback_ref as *mut xil::XTtcPs;

    // Clear timer interrupt status.
    let status_event = xil::XTtcPs_GetInterruptStatus(ttc);
    xil::XTtcPs_ClearInterruptStatus(ttc, status_event);
    xil::Xil_ExceptionEnable();
}

/// Timer interrupt handler for moving the alien, shooting, and other game logic.
/// See also [tick_handler](fn.tick_handler.html) and its line comments for details.
pub unsafe extern "C" fn tick_handler_1(callback_ref: *mut c_void) {
    // TODO: Write code here
    // If the game is still running, update game graphics etc.    
    
    if IS_WON != 1 && IS_LOST == 0 {
        
		// Alien handling (draw & check movement)
        handle_alien();
        
        // Handle bullet (draw & check movement)
        handle_bullet();
        
        // Draw player ship
		draw_player_ship();
        
		// Bullet impact!?
		check_impact();
	}
	
    // Draw SCORE & SHIELD
	draw_score();    

    // Call Assembly blinker function
    blinker();

    // End of your code

    // Clear timer interrupt status.
    let ttc = callback_ref as *mut xil::XTtcPs;
    let status_event = xil::XTtcPs_GetInterruptStatus(ttc);
    xil::XTtcPs_ClearInterruptStatus(ttc, status_event);
}

/// A custom panic handler for Cortex-A9.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // logs "panicked at '$reason', src/main.rs:27:4" to host stdout
    println64!("{}", info);

    loop {}
}

// Own debug notes that can be called from pixel.rs etc.
pub fn note(note: &str) {
    println64!("Note: {}", note);
}

unsafe fn init_game() {
	OPEN_CHANNEL = 0; // Currently open channel
	ALIEN_X = 0; // Alien X coordinate
	ALIEN_Y = 0; // Alien Y coordinate
	SHIP_X = 3; // Ship center X coordinate
	SHIP_Y = 7; // Ship center Y coordinate
	BULLET_X = 0; // Bullet X coordinate
	BULLET_Y = BULLET_HAVEN; // Bullet Y coordinate
	INCREMENT = 1; // Current direction of alien movement
	FREQUENCY = 10; // Movement frequency (difficulty)
	SCORE = 0; // Game SCORE
	SHIELD = 5; // Player ship "SHIELD" (0 is game over, every miss reduces one)
	IS_WON = 0; // flag for game won
	IS_LOST = 0; // flag for game lost

	// Clear screen
    for i in 0..8 {
		for j in 0..8 {
			set_pixel(i,j,0,0,0);
		}
	}
    
    note("New game initialized.");
}

// Checks if bullet is hitting alien, and sets SCORE/SHIELD accordingly
// Also checks SCORE and SHIELD for game end
unsafe fn check_impact() {
    if BULLET_Y == 0 {
        if ALIEN_X == BULLET_X {
            // Draw pixels on both side of the alien to indicate debris falling off after impact
            set_pixel(BULLET_X+1,BULLET_Y,255,255,255);
            set_pixel(BULLET_X-1,BULLET_Y,255,255,255);
            
            SCORE += 1; // hit score plus
            
            println64!("Score now: {}", SCORE);
            
            if SCORE > 2 {
                // game win
                win_game();
            }
        } else {
            SHIELD -= 1; // miss
            println64!("Shield now: {}", SHIELD);
            if SHIELD < 1 {
                // game over
                lose_game();
            }
        }
        BULLET_Y = BULLET_HAVEN; // "Hide" bullet from matrix
    }
}

// Draws player ship
// Note: ship old dots are reset when buttons are pressed
unsafe fn draw_player_ship() {
    set_pixel(SHIP_X, SHIP_Y,255,0,0); // center
    set_pixel(SHIP_X-1, SHIP_Y,255,0,0); // left
    set_pixel(SHIP_X+1, SHIP_Y,255,0,0); // right
    set_pixel(SHIP_X, SHIP_Y-1,255,0,0); // top
}

// Resets alien graphics, checks its movement direction and draws it to a new spot
unsafe fn handle_alien() {
    set_pixel(ALIEN_X,ALIEN_Y,0,0,0); // clear old alien location

    // Check if running to edges
    if ALIEN_X < 1 {
        INCREMENT = 1;
    }
    if ALIEN_X >= 7 {
        INCREMENT = -1;
    }

    // Move alien
    if INCREMENT > 0 {
        ALIEN_X += 1;
    } else {
        ALIEN_X -= 1;
    }

    // Draw alien
    set_pixel(ALIEN_X,ALIEN_Y,0,255,0);
}

//
unsafe fn handle_bullet() {
    // Draw bullet if on matrix
    if BULLET_Y < 10{
        set_pixel(BULLET_X,BULLET_Y,0,0,0); // bullet old location clear

        // Draw bullet (if on screen)
        if BULLET_Y > 0 && BULLET_Y < 10 {
            BULLET_Y -= 1;
        }
        set_pixel(BULLET_X, BULLET_Y, 0,0,255);
    }
}

// Draws SCORE & SHIELD dots
unsafe fn draw_score() {
	let score_spot = 0;
	let shield_spot = 7;
	// Clear old SCORE dots
    for i in (6..0).rev() {
		set_pixel(score_spot,usize::try_from(i).unwrap(),0,0,0);
	}
	// Draw SCORE dots
	for i in (0..6-SCORE).rev() {
		set_pixel(score_spot,usize::try_from(i).unwrap(),200,200,200);
	}
	// Clear old SHIELD
	for i in (1..6).rev() {
			set_pixel(shield_spot,usize::try_from(i).unwrap(),0,0,0);
		}
	// Draw SHIELD dots
	for i in (0..6-SHIELD).rev() {
		set_pixel(shield_spot,usize::try_from(i).unwrap(),50,100,150);
	}
}

// Set game to win state
// Draw win effect
unsafe fn win_game() {
    note("Game won!");
	IS_WON = 1;
	let r = 0;
	let g = 200;
	let b = 100;
	let mut x = 0;
    let mut y = 0;

    // Draw pixels to form a victory signal
	set_pixel(4,3,r,g,b);
	set_pixel(5,1,r,g,b);
	set_pixel(4,4,r,g,b);
	set_pixel(5,2,r,g,b);
	set_pixel(1,2,r,g,b);
	set_pixel(2,4,r,g,b);
    note("Winning pixels drawn.");
}

// Set game to lost state
// Draw lose effect
unsafe fn lose_game() {
    note("Game lost!");
	let mut r = 1;
	let mut g = 1;
	let mut b = 1;
	IS_LOST = 1;
    // Draw some colours to LED matrix to represent player ship "explosion"
	for i in 0..7 {
		for j in 0..7 {
			r+=2;
			g+=4;
			b+=6;
            // Prevent colour overflow
            if r > 240 {
                r = 0;
            }
            if g > 240 {
                g = 0;
            }
            if b > 240 {
                b = 0;
            }
			set_pixel(i,j,r,g,b);
		}
        open_line(i.try_into().unwrap());
	}
}