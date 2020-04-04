	nolist

;*	Macros for PIC 12XX Programming.
;	--------------------------------
;
;	proc/endproc macros define symbols that demarcate the beginning
;	and end of a procedure.	 Very useful for debugging.

_procnest	set	0		; procedure nesting counter

proc	macro	what			; begin procedure
what
start_#v(_procnest)what
_procnest	set	_procnest+1
	endm

endproc macro	what			; end procedure
_procnest	set	_procnest-1
end_#v(_procnest)what
	endm

;	Keep/unkeep - Bracket code that must stay together.
;	---------------------------------------------------
;
;	These are mostly for documentation.  When using
;	any of the skip instructions, it's important that
;	succeeding instructions not be moved, particularly
;	in the case of nested skips, such as:
;
;		btfss..
;		btfss...
;		goto...
;

keep	macro
	endm

unkeep	macro
	endm

;	Move immedate value to register - uses W

movif	macro	what,reg		; move immediate to register
	movlw	what
	movwf	reg
	endm

;	Move register-to-register - uses W

movff	macro	reg1,reg2		; move register to register
	movfw	reg1
	movwf	reg2
	endm

;	Set bank 0.
;
;	We re-enable the "not in zero bank"  warning.
;

bank0	macro				; set bank 0
	bcf	STATUS,RP0
	errorlevel +302
	endm

;	Set bank 1.
;
;	We disable the "not in zero bank"  warning.
;

bank1	macro				; set bank 1
	bsf	STATUS,RP0
	errorlevel -302
	endm

;	Decrement, branch if not zero.

dbnz	macro	what,where		; decrement reg, branch nonzero
	decfsz	what,f
	goto	where
	endm

;	Branch if bit set.

bbs	macro	reg,bit,where		; branch if bit set
	btfsc	reg,bit
	goto	where
	endm

;	Branch if bit clear.

bbc	macro	reg,bit,where		; branch if bit clear
	btfss	reg,bit
	goto	where
	endm

;	Logical shift right.

slrf	macro	reg,dest
	clrc
	rrf	reg,dest
	endm

;	Logical shift left.

sllf	macro	reg,dest
	clrc
	rlf	reg,dest
	endm

;	Negate (2's complement) W.

negw	macro
	sublw	0
	endm

;	Complement (1's complement) W.

notw	macro
	xorlw	0xff
	endm

;	Wait (stall) on bit clear.  That is, stall until bit set.

stallc	macro	reg,bit
	btfss	reg,bit
	goto	$-1
	endm

;	Wait (stall) on bit set.  That is, stall until bit clear.

stalls	macro	reg,bit
	btfsc	reg,bit
	goto	$-1
	endm

;	For 16-bit operations, big-endian register assignment is assmed.
;	if R holds the MSB of a 16-bit quantity, then R+1 holds the
;	LSB.


;	Move double register to register.

movfd	macro	reg1,reg2
	movfw	reg1
	movwf	reg2
	movfw	reg1+1
	movwf	reg2+1
	endm

;	Shift right double register (16 bit)

slrd	macro	reg
	clrc
	rrf	reg,f
	rrf	reg+1,f
	endm

;	Shift left double register (16 bit)

slld	macro	reg
	clrc
	rlf	reg+1,f
	rlf	reg,f
	endm

;	Delay 2 instruction cycles.

delay2	macro
	goto	$+1
	endm

	list
