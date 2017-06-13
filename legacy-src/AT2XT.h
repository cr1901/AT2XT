#ifndef AT2XT_H_
#define AT2XT_H_

//Useful Macros
#define TRUE 1
#define FALSE 0
#define PAUSE(_x) __delay_cycles(_x*2.1)

#define BIT(_x) (char) 1 << _x
/* This does not do what I expect (casts to int for some reason) */


//PORT1 is closest to voltage regulator on prototype board
#define CAPTURE_P2CLK_PIN 1
#define CAPTURE_P2CLK BIT1

#define PORT1_DATA_PIN 4
#define PORT1_DATA BIT4

#define PORT1_CLK BIT0

#define PORT2_DATA_PIN 3
#define PORT2_DATA BIT3

#define PORT2_CLK BIT2

#define DIRECTION_REG P1DIR
#define INPUT_REG P1IN
#define OUTPUT_REG P1OUT

#define TIMER_FRQ 100000

extern volatile char keycode_buffer[16];
extern volatile char buffer_head, buffer_tail;
extern volatile char timeout;

/* Might be necessary to handle bad keycodes in FSM. */
//extern volatile char BadKeys;


/* buffer_flush is... err, was a debugging function */
void BufferFlush();
void StartTimer(unsigned short clk_cycles);
void StopTimer();
void Stall(unsigned short clk_cycles);
void SendBytetoATKeyboard(char at_code);
void SendBytetoPC(char xt_code);
void InhibitComm();
void IdleComm();
//void SendRTS(PORTS p);
char ComputeParity(unsigned char shifted_byte);
unsigned char ReverseBits(unsigned char b);
void AT2XT_FSM();


#endif /*AT2XT_H_*/
