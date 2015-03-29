# AT-to-XT Keyboard Protocol Converter
This repository provides the source, schematics, and Gerber files that converts
the AT-keyboard protocol to the XT keyboard protocol. As XT keyboards are 
expensive (seriously, type in "PC XT keyboard" or "PC 5150 keyboard" in Ebay, 
this provides a cheaper alternative for someone willing to wait for PCB and 
parts. This circuit supports 101-key extended keyboards using the XT protocol, 
but older pre-386 systems may not know how to handle extended keys. The 
extended keycodes are based on a document from Microsoft that includes XT
keycodes for compatibility.

Currently, it is up to the user to set up their toolchain to compile the files
for programming an MSP430G2211 or compatible 14-pin DIP MSP430. I recommend the
former, if only because MSP430 is already overkill for this project and G2211
is a low-end model :P. I will add a Makefile when I can think of a solution that
allows Windows and users to use the same Makefile. In the interim, if using 
CCS, just add the .c and .h files to a project, and compile using -O2 or 
better.

The source code itself should be easy to port to other microcontrollers, 
except for the use of a __delay_cycles() intrinsic. I had no choice here, as 
using the timer for a software delay can lock the keyboard FSM to a single 
state.

Schematics are provided in DIPTrace ASCII format. PCB is provided using Gerber
Files and an N/C Drill File.
