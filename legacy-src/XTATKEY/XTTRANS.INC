;	AT-XT translation routine.
;	--------------------------
;
;	Given an AT code, return an XT code.
;	Enter with W = AT code
;	Exit with W = XT code or 0 if no mapping
;

	if	($ & 255) > 125		; it'll cross a page boundary
	org	($ & 0xff00) + 256
	endif				; move to the next page

TransATXT:
	andlw	127			; make sure we've just 7 bits.
	addwf	PCL,f

	retlw	0		; AT Code 00, Undefined
	retlw	0x43		; AT Code 01, Key 120
	retlw	0		; AT Code 02, Undefined
	retlw	0x41		; AT Code 03, Key 118
	retlw	0x3d		; AT Code 04, Key 114
	retlw	0x3b		; AT Code 05, Key 112
	retlw	0x3c		; AT Code 06, Key 113
	retlw	0x58		; AT Code 07, Key 123
	retlw	0		; AT Code 08, Undefined
	retlw	0x44		; AT Code 09, Key 121
	retlw	0x42		; AT Code 0a, Key 119
	retlw	0x40		; AT Code 0b, Key 117
	retlw	0x3e		; AT Code 0c, Key 115
	retlw	0x0f		; AT Code 0d, Key 16
	retlw	0x29		; AT Code 0e, Key 1
	retlw	0		; AT Code 0f, Undefined
	retlw	0		; AT Code 10, Undefined
	retlw	0x38		; AT Code 11, Key 60
	retlw	0x2a		; AT Code 12, Key 44
	retlw	0		; AT Code 13, Undefined
	retlw	0x1d		; AT Code 14, Key 58
	retlw	0x10		; AT Code 15, Key 17
	retlw	0x02		; AT Code 16, Key 2
	retlw	0		; AT Code 17, Undefined
	retlw	0		; AT Code 18, Undefined
	retlw	0		; AT Code 19, Undefined
	retlw	0x2c		; AT Code 1a, Key 46
	retlw	0x1f		; AT Code 1b, Key 32
	retlw	0x1e		; AT Code 1c, Key 31
	retlw	0x11		; AT Code 1d, Key 18
	retlw	0x03		; AT Code 1e, Key 3
	retlw	0		; AT Code 1f, Undefined
	retlw	0		; AT Code 20, Undefined
	retlw	0x2e		; AT Code 21, Key 48
	retlw	0x2d		; AT Code 22, Key 47
	retlw	0x20		; AT Code 23, Key 33
	retlw	0x12		; AT Code 24, Key 19
	retlw	0x05		; AT Code 25, Key 5
	retlw	0x04		; AT Code 26, Key 4
	retlw	0		; AT Code 27, Undefined
	retlw	0		; AT Code 28, Undefined
	retlw	0x39		; AT Code 29, Key 61
	retlw	0x2f		; AT Code 2a, Key 49
	retlw	0x21		; AT Code 2b, Key 34
	retlw	0x14		; AT Code 2c, Key 21
	retlw	0x13		; AT Code 2d, Key 20
	retlw	0x06		; AT Code 2e, Key 6
	retlw	0		; AT Code 2f, Undefined
	retlw	0		; AT Code 30, Undefined
	retlw	0x31		; AT Code 31, Key 51
	retlw	0x30		; AT Code 32, Key 50
	retlw	0x23		; AT Code 33, Key 36
	retlw	0x22		; AT Code 34, Key 35
	retlw	0x15		; AT Code 35, Key 22
	retlw	0x07		; AT Code 36, Key 7
	retlw	0		; AT Code 37, Undefined
	retlw	0		; AT Code 38, Undefined
	retlw	0		; AT Code 39, Undefined
	retlw	0x32		; AT Code 3a, Key 52
	retlw	0x24		; AT Code 3b, Key 37
	retlw	0x16		; AT Code 3c, Key 23
	retlw	0x08		; AT Code 3d, Key 8
	retlw	0x09		; AT Code 3e, Key 9
	retlw	0		; AT Code 3f, Undefined
	retlw	0		; AT Code 40, Undefined
	retlw	0x33		; AT Code 41, Key 53
	retlw	0x25		; AT Code 42, Key 38
	retlw	0x17		; AT Code 43, Key 24
	retlw	0x18		; AT Code 44, Key 25
	retlw	0x0b		; AT Code 45, Key 11
	retlw	0x0a		; AT Code 46, Key 10
	retlw	0		; AT Code 47, Undefined
	retlw	0		; AT Code 48, Undefined
	retlw	0x34		; AT Code 49, Key 54
	retlw	0x35		; AT Code 4a, Key 55
	retlw	0x26		; AT Code 4b, Key 39
	retlw	0x27		; AT Code 4c, Key 40
	retlw	0x19		; AT Code 4d, Key 26
	retlw	0x0c		; AT Code 4e, Key 12
	retlw	0		; AT Code 4f, Undefined
	retlw	0		; AT Code 50, Undefined
	retlw	0		; AT Code 51, Undefined
	retlw	0x28		; AT Code 52, Key 41
	retlw	0		; AT Code 53, Undefined
	retlw	0x1a		; AT Code 54, Key 27
	retlw	0x0d		; AT Code 55, Key 13
	retlw	0		; AT Code 56, Undefined
	retlw	0		; AT Code 57, Undefined
	retlw	0x3a		; AT Code 58, Key 30
	retlw	0x36		; AT Code 59, Key 57
	retlw	0x1c		; AT Code 5a, Key 43
	retlw	0x1b		; AT Code 5b, Key 28
	retlw	0		; AT Code 5c, Undefined
	retlw	0x2b		; AT Code 5d, Key 42
	retlw	0		; AT Code 5e, Undefined
	retlw	0		; AT Code 5f, Undefined
	retlw	0		; AT Code 60, Undefined
	retlw	0x56		; AT Code 61, Key 45
	retlw	0		; AT Code 62, Undefined
	retlw	0		; AT Code 63, Undefined
	retlw	0		; AT Code 64, Undefined
	retlw	0		; AT Code 65, Undefined
	retlw	0x0e		; AT Code 66, Key 15
	retlw	0		; AT Code 67, Undefined
	retlw	0		; AT Code 68, Undefined
	retlw	0x4f		; AT Code 69, Key 93
	retlw	0		; AT Code 6a, Undefined
	retlw	0x4b		; AT Code 6b, Key 92
	retlw	0x47		; AT Code 6c, Key 91
	retlw	0		; AT Code 6d, Undefined
	retlw	0		; AT Code 6e, Undefined
	retlw	0		; AT Code 6f, Undefined
	retlw	0x52		; AT Code 70, Key 99
	retlw	0x53		; AT Code 71, Key 104
	retlw	0x50		; AT Code 72, Key 98
	retlw	0x4c		; AT Code 73, Key 97
	retlw	0x4d		; AT Code 74, Key 102
	retlw	0x48		; AT Code 75, Key 96
	retlw	0x01		; AT Code 76, Key 110
	retlw	0x45		; AT Code 77, Key 90
	retlw	0x57		; AT Code 78, Key 122
	retlw	0x4e		; AT Code 79, Key 106
	retlw	0x51		; AT Code 7a, Key 103
	retlw	0x4a		; AT Code 7b, Key 105
	retlw	0x37		; AT Code 7c, Key 100
	retlw	0x49		; AT Code 7d, Key 101
	retlw	0x46		; AT Code 7e, Key 125
	retlw	0		; AT Code 7f, Undefined
