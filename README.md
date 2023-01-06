# EmbeddedAlienShooting
Project work exercise for creating a simple shooting game on a Pynq development board

This project work is part of a university course Introduction to Embedded Systems course from Autumn semester 2022. The task was to make an "alien shooter" game on an embedded development board with a LED matrix attached. Used development board is Pynq-Z1 which uses Zynq-7000 SoC chip. Zynq includes ARM Cortex-A9 processor (32-bit) and Artix-7 family FPGA. The "Magic" RGB LED Matrix was, according to provided documentation, Colors Shield which pairs M54564 with DM163 driver.

The project was first coded in C language on top of a template, that was provided by the course. The template provided base structures for the project plus some preset configurations for the development board. Initializing the device, setting up the LED matrix and all game logic had to be done ownself, while interrupt handling was basically provided ready. As the first bonus task there was to do an additional LED blinker function in ARM Assembly, and as a larger bonus task there was to make the whole project again in Rust language.

I did the main task and the both bonus tasks. Rust provided some challenges, since I had never had any experience with the language and it was only a week time to complete it for me after the C part (with which I was already familiar with and for which I had time for up to six weeks). For the C code part the IDE used with was Xilinx SDK. For Rust it was coded with Visual Studio Code.

At least for now, this repository only contains the Rust parts of my code (with Assembly inline). The C language source code is currently "trapped" inside the university network drive, to which I have no free access from home, and after the course ended I have not had much business to university computers. Maybe I can some day salvage that too.

The original Rust project template can be found here: https://github.com/hegza/alien-shooter-template-rs
I'm not sure if the C project template exists anywhere in public.

The repository is does not contain all the files necessary for running the application on Pynq board. The files included are only my own custom code files - this is not intended as a project for downloading for running it on another development board, but it's just for storing code for myself. Also for your own sake, please don't copy the code for your own course work in case the same project is done again another year - there will likely be issues both with learning and running the code, especially as there are critical parts missing. :)
