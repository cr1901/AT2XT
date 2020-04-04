	title	"PC AT/PS/2 to XT keycode translator."
	list	p=12F629
	radix	dec
	noexpand			; I like neat listings

	include "p12f629.inc"
	include "maclib.inc"

;*	AT/PS2 to XT Keyboard Translator.
;	---------------------------------
;
;	Version 0.90  Chuck Guzis, June, 2009
;	Copyright by Charles P. Guzis, all rights reserved.
;
;	Chuck Guzis retains all rights to this code, but gives permission
;	to use and modify it for non-commercial, non-profit applications,
;	provided that this copyright notice is reproduced in its entirety.
;
;	This translator uses a small PIC12F629 microcontroller to translate
;	an AT or PS/2 keyboard interface to that of the PC XT.	Shift-lock
;	status is noted and the keyboard LEDs are illuminated accordingly.
;
;	The electrical interface to the 12F629 is very simple:
;
;	Pin 1 = +5 supply (should have a 47 uF decoupling capacitor to Gnd
;	Pin 2 = Clock from the PC XT host
;	Pin 3 = Data from the PC XT host
;	Pin 4 = unused
;	Pin 5 = AT keyboard clock  (pulled high by 4.7K to +5)
;	Pin 6 = AT keyboard data   (pulled high by 4.7K to +5)
;	Pin 7 = AT clock pulldown (a 1N4148 diode is connected between
;		Pin 7 (cathode end) and pin 5 (anode end)
;	Pin 8 = Gnd
;
;	The 1N4148 diode isn't critical--any general-purpose signal
;	diode will work.  It's there to make GP0 behave as an open-
;	drain driver.  Since the PIC is a CMOS device, the extra
;	diode drop will have no significant effect on operation.
;
;	One could enable the MCLEAR function on the PIC and tie pin 4 to
;	the RESET line from the XT host, but since the original XT keyboard
;	doesn't use this signal, neither did I.
;
;	The XT reset line is not used, as the XT BIOS asserts reset by
;	holding the host clock low for at least 20 mS.
;
;	Just so you don't have to look it up, here are the pinouts
;	for both the XT (5 pin DIN) and PS/2 (6 pin mini-DIN) conectors:
;
;	PS/2 connector:
;		1 - Data
;		3 - Gnd
;		4 - +5
;		5 - Clock
;
;	XT Connector:
;		1 - Clock
;		2 - Data
;		3 - Reset, not used
;		4 - Gnd
;		5 - +5
;
;	A 32-byte buffer is implemented for received keyboard data.
;	This is actually twice the size of the AT keyboard buffer.
;	Extraneous code bytes E0 and E1 are ignored from the AT keybaord.
;
;	A 12F675 can be used in this application, but see the commented-out
;	line in the "Setup" routine.
;

	__CONFIG	_MCLRE_OFF & _CP_OFF & _WDT_OFF & _INTRC_OSC_NOCLKOUT  ;Internal osc.
	subtitle	"Symbol definitions"
	page

;	Our GPIO bit assignments by position.

bnKeyClockPD	equ	0		; AT key  pulldown  (GPIO 0, pin 7)
bnKeyData	equ	1		; AT keyboard data  (GPIO 1, pin 6)
bnKeyClock	equ	2		; AT Keyboard clock (GPIO 2, pin 5)
bnReset		equ	3		; MCLEAR, unused    (GPIO 3, pin 4)
bnHostData	equ	4		; XT host data	    (GPIO 4, pin 3)
bnHostClock	equ	5		; XT host clock	    (GPIO 5, pin 2)

;	Same as the above, but by bit values.

bvKeyClockPD	equ	1<<bnKeyClockPD ; AT key pulldown   (GPIO 0, pin 7)
bvKeyData	equ	1<<bnKeyData	; AT keyboard data  (GPIO 1, pin 6)
bvKeyClock	equ	1<<bnKeyClock	; AT keyboard clock (GPIO 2, pin 5)
bvReset		equ	1<<bnReset	; MCLEAR,	    (GPIO 3, pin 4)
bvHostData	equ	1<<bnHostData	; XT host data	    (GPIO 4, pin 3)
bvHostClock	equ	1<<bnHostClock	; XT host clock	    (GPIO 5, pin 2)

;	Bits relating to keyboard state in rATFlags.
;
;	Observe that the LED (shift lock) keys are in the precise
;	position that the AT keyboard ED command expects them.

bnATRelease	equ	7		; we saw a key-up
bnATScrollLock	equ	0		; scroll lock
bnATNumLock	equ	1		; numeric lock
bnATCapsLock	equ	2		; caps lock

bvATRelease	equ	1<<bnATRelease		; if key-up seen
bvATScrollLock	equ	1<<bnATScrollLock	; scroll lock
bvATNumLock	equ	1<<bnATNumLock		; numeric lock
bvATCapsLock	equ	1<<bnATCapsLock		; caps are locked

;	Register holding flags for the "send byte" code in the
;	interrupt service code.

bnATSendReq	equ	0		; if set, there's a character in
					; rATSendByte that needs to go
bnACKSeen	equ	1		; ACK isn't put into the queue;
					; rather when received, this flag
					; is set.

;	Input queue area starts at 0x40 and goes through 0x5f

QUEUE_SIZE	equ	32		; cells to use in queue

;	XT scancodes for special keys.

XTSC_CAPSLOCK	equ	0x3a		; Caps lock
XTSC_SCRLOCK	equ	0x46		; Scroll lock
XTSC_NUMLOCK	equ	0x45		; Numeric lock
XTSC_OKAY	equ	0xaa		; Scan code for "Diagnostics Passed"

;	AT command codes.

ATSC_SETLED	equ	0xed		; set LEDs
ATSC_RESET	equ	0xff		; reset
ATSC_ACK	equ	0xfa		; acknowledge
ATSC_OK		equ	0xaa		; diagnostics okay

	subtitle	"Data area (register) layout"
	page

;	Register file definitions.

	cblock	0x20

	rXTByte		:1		; byte to send - used by SendXTByte
	rXTBitCount	:1		; bits left to send

	rATChar		:1		; character returned by ReadKey
	rATFlags	:1		; flags set by AT codes

	rTemp1		:1		; Scratch reg 1
	rTemp2		:1		; Scratch reg 2

;	Interrupt routine registers.

	rIntSaveW	:1		; Savearea for W
	rIntSaveS	:1		; Savearea for status word
	rIntSaveFSR	:1		; FSR save
	rIntKCount	:1		; keyboard data bit counter
	rIntKData	:1		; keyboard code accumulator
	rIntTemp	:1		; temporary for the ISR
	rATSendByte	:1		; byte to send to keyboard
	rATSendFlags	:1		; flags for the send routine

;	Keyboard buffer pointers.

	rKeyQIn		:1		; in
	rKeyQOut	:1		; out

	endc

;	The keyboard buffer.

	cblock	0x40

	rKeyQFirst	:QUEUE_SIZE	; circular queue

	endc

;*	Boot entry.
;	==========


Start	org	0x0000		;program starts at location 000
	call	SetUp
	goto	Main			; start the routine

	subtitle	"Interrupt servicer"
	page

;	Interrupt servicing routine.
;	----------------------------
;
;	We interrupt on the AT keyboard clock bit high-to-low transition.
;	Spurious interrupts are possible, so check and discard them.
;

	org	0x0004

	proc	IntServ
	movwf	rIntSaveW
	swapf	STATUS,w		; movfw changes the Z status so can't use
	bank0
	movwf	rIntSaveS		; save the status word

;	When we come in here, we've triggered on a high-to-low transition of the
;	AT keyboard clock.  If the keyboard is sending data to us, the clock
;	will be low and the data line will be low also (start bit).  If we're
;	sending data to the keyboard, the data line will be low because we
;	pulled it low.


	keep
	btfss	GPIO,bnKeyData		; if keyboard data high, ignore
	btfsc	GPIO,bnKeyClock		; if keyboard clock high, ignore
	goto	IntServExit		; ignore the interrupt
	unkeep

;	See if we're asking to send a byte.  If we get a collision,
;	(a byte coming in when we're trying to send), it's not fatal.

	bbs	rATSendFlags,bnATSendReq,IntServ20	; if sending

	movif	8,rIntKCount		; bit counter
	clrc				; clear carry
	clrf	rIntKData		; clear accumulated bytes

;	Wait for the clock to transit high, then low (skip the start bit).

IntServ2:
	stallc	GPIO,bnKeyClock		; wait for clock to  rise
	stalls	GPIO,bnKeyClock		; and wat for clock to fall again

;	Get the data bits

	keep
	btfsc	GPIO,bnKeyData		; skip if a 0 bit
	setc				; set a 1 bit
	unkeep
	rrf	rIntKData,f		; the keyboard data bits
	dbnz	rIntKCount,IntServ2	; for next bit

;	We still have two bits to go, parity and stop.	We skip them.

	stallc	GPIO,bnKeyClock		; wait for clock to rise
	stalls	GPIO,bnKeyClock		; and fall again  (skip parity bit)
	stallc	GPIO,bnKeyClock		; wait for clock rise
	stalls	GPIO,bnKeyClock		; and fall again (skip stop bit)

;	We special-case ACK.  Because it occurs after a command has been
;	sent, we can't put it into the queue or the routine sending the
;	command may never see it.

	movfw	rIntKData		; get data just received
	xorlw	ATSC_ACK		; see if ACK
	bnz	IntServ14		; skip.
	bsf	rATSendFlags,bnACKSeen	; say we saw it
	goto	IntServExit		; don't put it in the queue

;	Now, add the assembled byte to the buffer.  Note that if IN+1 == OUT
;	there's no room in the buffer, so we toss the keystroke.

IntServ14:
	incf	rKeyQIn,w		; advance
	andlw	0x5f			; enforce a wrap-around
	movwf	rIntTemp		; save it
	sublw	rKeyQOut		; see if equal
	bz	IntServExit		; if queue full, drop the character

;	N.B.:  If we see that we have problems with overflow, we can drive the
;	keyboard clock low and essentially shut things off until we have a
;	chance to empty the queue.

	movff	FSR,rIntSaveFSR		; save current FSR
	movff	rIntKData,INDF		; store the byte
	movff	rIntTemp,rKeyQIn	; update the "in" pointer
	movff	rIntSaveFSR,FSR		; restore FSR
	goto	IntServExit		; all done

;	This is where we send code to the AT Keyboard.

IntServ20:
	bcf	rATSendFlags,bnACKSeen	; say we haven't seen ACK
	bcf	rATSendFlags,bnATSendReq	; clear request flag
	movif	8,rIntKCount		; how many bits to send out
	clrf	rIntTemp		; parity counter
IntServ22:
	stallc	GPIO,bnKeyClock		; wait for clock to rise
	rrf	rATSendByte,f		; get a bit
	bc	IntServ24		; if a 1
	bcf	GPIO,bnKeyData		; set a zero
	goto	IntServ26		; keep going

IntServ24:
	bsf	GPIO,bnKeyData		; set a 1
	incf	rIntTemp,f		; count parity
IntServ26:
	stalls	GPIO,bnKeyClock		; wait for clock to fall
	dbnz	rIntKCount,IntServ22	; for next bit

;	We need to add the parity bit.	If we have an even number of
;	1 bits in the data stream, we add a 1 bit, otherwise, a zero.

	stallc	GPIO,bnKeyClock		; wait for clock to rise
	keep
	btfsc	rIntTemp,0		; skip if even number of bits
	bcf	GPIO,bnKeyData		; add 1
	btfss	rIntTemp,0		; skip if odd number of bits
	bsf	GPIO,bnKeyData		; add 0
	unkeep

	stalls	GPIO,bnKeyClock		; wait for clock to fall

;	Now, float the data line.

	nop
	nop				; kill 2 uS

	bank1
	bsf	TRISIO,bnKeyData	; set data line to input
	bank0

	stallc	GPIO,bnKeyClock		; wait for high clock

;	Wait for the device to bring data low.

	clrf	TMR1L
	movif	(256-5),TMR1H		; set about 20 msec
	bcf	PIR1,TMR1IF		; clear interupt flag
IntServ32:
	bbs	PIR1,TMR1IF,IntServ40	; if timeout
	bbs	GPIO,bnKeyData,IntServ32; stall until data low

;	Data is low, wait for clock low.

IntServ34:
	bbs	PIR1,TMR1IF,IntServ40	; if timeout
	bbs	GPIO,bnKeyClock,IntServ34	; wait around

;	Now wait for a high clock.

IntServ36:
	bbs	PIR1,TMR1IF,IntServ40	 ; if timeout, just quit
	bbc	GPIO,bnKeyClock,IntServ36 ; wait for rising clock

;	The keyboard responded as expected, if we get here.


;	We're done -- the keyboard should respond with an ACK.

IntServ40:

;	Return from interrupt.

IntServExit:
	bcf	INTCON,INTF		; clear the old interrupt
	bsf	INTCON,INTE		; re-enable interrupts
	swapf	rIntSaveS,w
	movwf	STATUS			; restore status
	swapf	rIntSaveW,f
	swapf	rIntSaveW,w
	retfie				; return from interrupt
	endproc IntServ

	subtitle	"Scan code translation"
	page

;*	Keyboard translation lookup.
;
;	We put the translation subroutine here, as there's no chance of
;	crossing a page boundary.

	include "xttrans.inc"		; keycode translator

	subtitle	"Read/write key code routines"
	page

;*	ReadKey - Read a key from the queue.
;	------------------------------------
;
;	If there are no keystrokes in the buffer, stall until
;	some arrive.  Returns next keystroke in rATChar and Z clear,
;	or Z set if no character.
;

	proc	ReadKey
ReadKey2:
	movfw	rKeyQOut
	movwf	FSR			; prepare for fetching
	xorwf	rKeyQIn,w
	bz	ReadKey4		; return Z set if empty

;	Retrieve the keystroke.

	movff	INDF,rATChar		; get the key
	incf	rKeyQOut,w		; advance
	andlw	0x5f			; wrap-around
	movwf	rKeyQOut		; update pointer
	clrz				; say we have one
ReadKey4:
	return
	endproc ReadKey

;*     SendXTBit - Send a bit to the XT keyboard port.
;      -----------------------------------------------
;
;	Note that bits are clocked by the XT on the high-to-low
;	transition of the clock.  This routine strobes the bit
;	onto the XT data line, waits for 30 usec, then drops
;	the clock line for 30 usec, then raises it and exits.
;
;	On entry, the data bit is in C (carry).
;

	proc	SendXTBit
	keep
	skpnc				; skip if cy clear
	bsf	GPIO,bnHostData
	skpc				; skip if cy set
	bcf	GPIO,bnHostData
	unkeep
	movlw	50/4			; 50 usec
	bcf	GPIO,bnHostClock	; drop the clock line
	movlw	50/4
	call	ShortDelay
	bsf	GPIO,bnHostClock	; raise clock
	return
	endproc SendXTBit

;*	SendXTByte - Send a byte to the XT.
;	-----------------------------------
;
;	Inserts a start bit, waits for XTClock and XTData to go high before
;	sending.
;
;	Byte to send is in rXTByte.
;
;	We need to frame things accurately, so data bit transitions are made
;	in the center of the clock-low bit cell.
;
;	The XT interface is very simple.  Bits are shifted into an 74LS322 shift
;	register.  When the start bit reaches the high-order bit of the shift
;	register, it sets an interrupt, disables shifting and pulls data low one
;	shift time later.  In other words, the start bit is shifted out.
;	The bit order is LSB first for 8 data bits.  After that, the interface
;	is essentially blind until the PC has picked up the character.
;
;	Many keyboards start out by strobing a low "pseudo-start"  bit, then
;	follow with a high "real start".  Apparently, this results in more
;	stable performace.  We'll do the same thing.
;
;	Clock is held low by the PC as an inhibit--the keyboard should (and
;	cannot) send data until both data and clock lines have been allowed
;	to go high.
;
;	The actual data rate only needs to be slower than the processor clock,
;	as a conditioning circuit formed by two flip-flops is clocked from
;	that to avoid any problems that might arise with ringing.
;

	proc	SendXTByte

;	Wait for high level on XTClock--and XTData.


SendXTByte2:
	keep
	btfsc	GPIO,bnHostClock	; loop if clock low
	btfss	GPIO,bnHostData		; loop if data low
	goto	SendXTByte2
	unkeep

;	Put XTClock and XTData into output mode with clock and data high.

	bsf	GPIO,bnHostClock
	bsf	GPIO,bnHostData
	bank1
	bcf	TRISIO,bnHostClock	; enable clock output
	bcf	TRISIO,bnHostData	; enable data output
	bank0

;	Start off with a 0 bit, then a 1 bit.

	keep
	clrc
	call	SendXTBit
	setc
	call	SendXTBit
	unkeep

;	Now, send the remaining 8 bits of the scan code.

	movif	8,rXTBitCount		; number of bits to send
SendXTByte4:
	rrf	rXTByte,f
	call	SendXTBit		; send, starting with the LSbit
	dbnz	rXTBitCount,SendXTByte4 ; loop

;	Raise the data line, pause, then set the clock and data to input.

	bsf	GPIO,bnHostData

;	At this point, the PC will be driving data low until the interrupt
;	has been serviced, so set data and clock to input.

	bank1
	bsf	TRISIO,bnHostData	; disable data
	bsf	TRISIO,bnHostClock	; disable clock output
	bank0

	return
	endproc SendXTByte

;*	SendATByte - Send a byte to the keyboard.
;	-----------------------------------------
;
;	This is tricky.	 We have to take the AT keybaord
;	clock low for more than 60 uS, drop the data line, then
;	let the clock go.
;
;	The interrupt routine will pick up when the next clock hits,
;	as long as rATSendByte is nonzero--whence our kludge.
;
;	W has the data we want to send.
;

	proc	SendATByte

	bcf	INTCON,INTE		; inhibit clock interrupt
	movwf	rATSendByte		; save it for the interrupt routine
	stallc	GPIO,bnKeyClock		; stall until keyboard clock hich

	bcf	GPIO,bnKeyClockPD	; pull the clock low

;	Stall with the clock low to prevent keyboard from starting a new
;	character.  We also have to mask the clock interrupt, since we're
;	pulling the clock low.

	movlw	100/4
	call	ShortDelay		; stall for 100 uS
	bcf	INTCON,T0IF		; clear interrupt flag which was set

	bank1
	bcf	TRISIO,bnKeyData	; set key data to output
	bank0

	bcf	GPIO,bnKeyData		; bring data low
	movlw	30/4			; more than 1 bit time
	call	ShortDelay
	bcf	rATSendFlags,bnACKSeen	; clear the "ACK seen"	flag
	bsf	rATSendFlags,bnATSendReq	; say we have a byte
	bsf	INTCON,INTE		; enable interrupt
	bsf	GPIO,bnKeyClockPD	; let keyboard clock float high

;	We stall until the byte gets sent.  We wait for an ACK.

	clrf	TMR1L
	movif	(256-5),TMR1H		; set about 20 msec
	bcf	PIR1,TMR1IF		; clear interupt flag
SendATByte4:
	bbs	PIR1,TMR1IF,SendATByte8 ; if timeout
	bbc	rATSendFlags,bnACKSeen,SendATByte4	; loop until ACK set

;	We timed out here, so we set the data to input.

SendATByte8:
	bank1
	bsf	TRISIO,bnKeyData	; set data to input--in case it's not
	bank0

	return				; all done

	endproc SendATByte

	subtitle	"Miscellaneous routines."
	page

;*	ShortDelay - Delay less than 1 msec.
;	------------------------------------
;
;	On entry, W has delay time in 4 microsecond units.
;	Resolution is about 4-8 uS.
;

	proc	ShortDelay

	sublw	255
	movwf	TMR0
	bcf	INTCON,T0IF		; clear interrupt flag
	stallc	INTCON,T0IF		; stall until interrupt set
	return
	endproc ShortDelay


;*	CheckLockKey - Check for Caps, Num and Scroll Lock.
;	---------------------------------------------------
;
;	Just toggles the state of the LEDs.  The XT scan code (make)
;	is in W.
;

	proc	CheckLockKey
	movwf	rTemp1			; save the code
	keep
	btfsc	rTemp1,7		; if it's not a break code
	return
	unkeep

	movlw	XTSC_CAPSLOCK
	xorwf	rTemp1,w
	bnz	CheckLockKey2		; if not capslock
	movlw	bvATCapsLock
	xorwf	rATFlags,f		; toggle caps lock
	goto	UpdateLED		; go update LED status

CheckLockKey2:
	movlw	XTSC_SCRLOCK
	xorwf	rTemp1,w
	bnz	CheckLockKey4		; if not scroll lock
	movlw	bvATScrollLock
	xorwf	rATFlags,f		; toggle scroll lock
	goto	UpdateLED		; update the LED status

CheckLockKey4:
	movlw	XTSC_NUMLOCK
	xorwf	rTemp1,w
	bnz	CheckLockKey10		; if not numlock
	movlw	bvATNumLock
	xorwf	rATFlags,f		; toggle num lock
	goto	UpdateLED		; update the LED status

CheckLockKey10:
	return				; all done

	endproc CheckLockKey

;*	UpdateLED - Update the LEDs.
;	---------------------------
;
;	Done all at once.
;

	proc	UpdateLED
	movlw	ATSC_SETLED		; set/reset LED
	call	SendATByte		; send the command
	movfw	rATFlags		; get LED status
	andlw	7			; isolate
	call	SendATByte		; return state
	return				; exit

	endproc UpdateLED


;*	PollHost - See what the XT is doing.
;	------------------------------------
;
;	At some point, the XT host will drop the keyboard clock
;	line for 20 mS or more.	 When this happens, we need to
;	simulate a keyboard reset.
;
;	We also clear all of our keyboard flags (and eventually reset
;	the AT keyboard).
;

	proc	PollHost
	bbs	GPIO,bnHostClock,PollHost10	; exit if clock high

;	Clock has gone low; time it.

	clrf	TMR1L
	movif	(256-5),TMR1H		; set about 10 msec
	bcf	PIR1,TMR1IF		; clear interupt flag
	stallc	GPIO,bnHostClock	; stall unit XT Clock high

;	Clock is high again--see for how long.

	bbc	PIR1,TMR1IF,PollHost10	 ; not long enough, just exit

;	Send out a "Diagnostics passed byte"

	movif	XTSC_OKAY,rXTByte
	call	SendXTByte		; send it

;	Reset the AT keyboard also.

	movlw	ATSC_RESET
	call	SendATByte		; send a reset to the AT keyboard

	clrf	rATFlags		; clear flags
	call	UpdateLED		; update LEDs
PollHost10:
	return

	endproc PollHost

	subtitle	"Main routine"
	page

;*	Main Loop.
;	----------
;
;	Here's the control flow:
;
;	a.  Check the host--has the clock been pulled low for more than
;	    10 mS.?  If so, issue a reset to the AT keyboard and clear
;	    our shift LED status and send an 0xaa to the host.
;	b.  See if an AT keystroke is available.  If so, look at it--
;	    discard E0 and E1 keystrokes, as they don't apply to the
;	    smaller keyboard of the XT.	 Discard any other keycodes
;	    with bit 7 set, as they're probably messages (such as AA)
;	    that the XT host can't process.  Note the occurrence of F0
;	    (key up) keystrokes.  Translate whatever's left.
;	c.  If any of the translated keycodes are shift-lock releases
;	    update the corresponding LED.
;	d.  Send the keystroke (with key-up modifier) to the host.
;	e.  Go to (a)
;

	proc	Main

Main2:
	call	PollHost		; poll the host interface

;	Main loop.  Sit around and cycle until there's data in the
;	AT keyboard receive buffer.

Main4:
	call	ReadKey			; read something from the buffer
	bz	Main2			; no data
	movfw	rATChar			; get the key
	bbc	rATChar,7,Main6		; if not a special
	xorlw	0xf0			; it's a key-up
	keep
	skpnz
	bsf	rATFlags,bnATRelease	; key is released
	unkeep
	goto	Main4			; get next keystroke

;	Something resembling a normal keycode has arrived.

Main6:
	movfw	rATChar
	call	TransATXT		; translate
	iorlw	0			; test for zero
	bz	Main8			; skip if undefined

	keep
	btfsc	rATFlags,bnATRelease
	iorlw	128			; add the release flag
	unkeep

	movwf	rXTByte

	call	CheckLockKey		; check for lock key
Main7:
	call	SendXTByte		; send it
Main8:
	bcf	rATFlags,bnATRelease	; clear key-up flag
	goto	Main2			; keep going

	endproc Main

	subtitle	"Initialization code"
	page

;*	Setup - Boot setup tasks.
;	-------------------------
;
;	Set up I/O ports, timers and interrupts.
;	Initialize variables.
;

	proc	SetUp

	bank1

;	Set up TIMER0 as a simple counter.

	movif	b'00010001',OPTION_REG	; /4 counter, internal clock
					; to TMR, enable pullups, GP2INT falling

;	Just in case there's no keyboard or PC attached, we enable weak
;	pullups on all pins to keep them from floating.

	movif	(bvKeyClock+bvKeyData+bvReset+bvHostClock+bvHostData),WPU

;	Initially, the only outputs that are enabled are for the AT clock
;	pulldown. Everything else is input.

	movif	(bvKeyClock+bvKeyData+bvHostData+bvHostClock),TRISIO

;	Calibrate the internal oscillator

	call	0x3ff		;get the calibration value
	movwf	OSCCAL		;calibrate oscillator

;	The following line should be uncommented if a 12F675 is being used.
;	It disables the AD converter and sets GP2 to digital mode.

;	 clrf	 0x1f			; ANSEL (12F675 only)

	bank0

	movif	7,CMCON			; disable comparator

;	Initialize variables.

	movif	rKeyQFirst,rKeyQIn
	movwf	rKeyQOut		; set IN=OUT on keyboard queue
	clrf	rATFlags		; LED status, key-up, etc.
	clrf	rATSendFlags		; clear flags
	clrf	rATSendByte		; clear the host-to-keyboard flag

;	Set the outputs to a known state.

	movif	bvKeyClockPD+bvHostData+bvHostClock,GPIO

;	Timer0 is used as a sub-1 mS timer, while timer1 is used for
;	millisecond intervals (mostly as a deadman).

	movif	b'00100001',T1CON	; TIMER1 internal, osc, /4, enabled
	movif	(1<<GIE)+(1<<INTE),INTCON	; GP2 interrupts
	bsf	T1CON,TMR1ON		; start timer 1

	return				; exit
	endproc Setup

	end
