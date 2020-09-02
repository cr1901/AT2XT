#include <msp430g2211.h>
//#include <math.h>
#include "ATcommands.h"
#include "AT2XT.h"

//Debug variables
#ifdef __DEBUG__
	//char send_delay = FALSE;
	volatile int countera = 0;
	volatile int counterb = 0;
	volatile int counterc = 0;
	volatile int counterd = 0;
	int test_byte = 0;

#endif

/* https://lists.llvm.org/pipermail/llvm-dev/2018-May/123600.html */
#ifdef __clang__
#define ISR_BEGIN(id) __attribute__((interrupt(0)))
#else
#define ISR_BEGIN(id) __interrupt_vec(id)
#endif

#ifdef __clang__
#define ISR_END(name)                                                          \
    __attribute__((section("__interrupt_vector_" #name), aligned(2))) void (   \
        *__vector_##name)(void) = name;
#else
#define ISR_END(name)
#endif

/* Extern definitions */
volatile char keycode_buffer[16] = {'\0'};
volatile char buffer_tail = 0, buffer_head = 0;


volatile short P1In_buffer = 0;
volatile short P1Out_buffer = 0;
volatile char BadKeys = 0;
volatile char HostMode = FALSE;
volatile char DeviceACK = FALSE;
volatile char timeout = FALSE;


void SendXTBit(char bit)
{
	OUTPUT_REG &= ~PORT2_DATA;
	OUTPUT_REG |= (bit << PORT2_DATA_PIN); //Shift into Port 2 data
	OUTPUT_REG &= ~PORT2_CLK;
	PAUSE(55);
	OUTPUT_REG |= PORT2_CLK;
}

//This routine takes in all bytes sent by the keyboard.
void SendBytetoPC(char xt_code)
{
	char num_bits = 0;

	#ifdef __DEBUG__
		const char test_keycode = 0x25; //'K' scancode
		//xt_code = test_keycode;
		#ifdef __XT_TEST_KEYCODE__
			xt_code = test_keycode;
		#endif
	#endif
	//Check that start/stop bits are good



	//1
	//#ifdef __DEBUG__
	//test_byte = 0;
	while(((P1IN & PORT2_CLK) == 0) || ((P1IN & PORT2_DATA) == 0))
	{
		//test_byte++;
		 /* Wait for both CLK and DATA to go high */
	}

	//TACCTL0 &= ~CCIE;

	//Set to output (both lines not being driven)
	OUTPUT_REG |= PORT2_CLK;
	OUTPUT_REG |= PORT2_DATA;

	//2
	DIRECTION_REG |= (PORT2_CLK + PORT2_DATA);
	SendXTBit(0);
	SendXTBit(1);

	for(num_bits = 0; num_bits <= 7; num_bits++)
	{
		SendXTBit((xt_code & 0x01)); /* Send data... */
		xt_code = xt_code >> 1;
	}

	//8
	OUTPUT_REG |= PORT2_DATA;

	//9
	DIRECTION_REG &= ~(PORT2_CLK + PORT2_DATA);
}

void SendBytetoATKeyboard(char at_code)
{
	P1Out_buffer = (short) at_code;
	//P1Out_buffer = ((P1Out_buffer << 1) | ComputeParity(P1Out_buffer));
	P1Out_buffer = (P1Out_buffer | (ComputeParity(P1Out_buffer) << 8));
	P1IE &= ~PORT1_CLK; //Disable interrupts

	/* Wait for clock to go high. */
	while((P1IN &PORT1_CLK) == 0);

	/* Cannot use the 12000 or 100000 Hz timer here...
	 * it is possible to lock the microcontroller in
	 * Stall() (especially when LEDs are being serviced)!
	 * Use PAUSE() macro instead, which delays
	 * by a number of cycles. */
	InhibitComm();
	PAUSE(110); /* Delay 100 microseconds or so... */

	P1OUT &= ~PORT1_DATA;
	PAUSE(33);

	P1OUT |= PORT1_CLK;
	DIRECTION_REG &= ~PORT1_CLK;
	P1IFG &= ~PORT1_CLK;	/* Clear pending interrupt */
	P1IE |= PORT1_CLK; //Re-enable interrupts

	HostMode = TRUE;
	DeviceACK = FALSE;
	StartTimer(TIMER_FRQ/50); /* Wait for DeviceACK or 20ms Timeout */

	while((DeviceACK == FALSE) && !timeout)
	{

	}
	StopTimer();
	HostMode = FALSE;
}

//AT Host
void SendBytetoAT()
{

}

void ResetXTKeyboard()
{

}


void StartTimer(unsigned short clk_cycles)
{
	TACCR0 = clk_cycles;
	TACCTL0 &= ~CCIFG; /* Clear pending interrupt. */
	timeout = FALSE;
}

void Stall(unsigned short clk_cycles)
{
	StartTimer(clk_cycles);
	while(!timeout);
	/* Interrupt will call StopTimer. */
	//TACCR0 = 0;
}

void StopTimer()
{
	TACCR0 = 0;
	TACCTL0 &= ~CCIFG; /* Clear pending interrupt. */
}

void InhibitComm()
{
	OUTPUT_REG &= ~PORT1_CLK; //Clk should be low
	OUTPUT_REG |= PORT1_DATA; //Data should be high
	DIRECTION_REG |= PORT1_CLK + PORT1_DATA; //Make these ports outputs
}

void IdleComm()
{
	OUTPUT_REG |= PORT1_CLK;
	OUTPUT_REG |= PORT1_DATA;
	DIRECTION_REG &= ~(PORT1_CLK + PORT1_DATA);
}


void BufferFlush()
{
	char count;
	for(count = 0; count < 16; count++)
	{
			keycode_buffer[count] = 0x01;
	}
	buffer_head = buffer_tail = 0;
}

//http://www-graphics.stanford.edu/~seander/bithacks.html#ParityParallel
char ComputeParity(unsigned char shifted_byte)
{
	char i, num_ones;
	for(i = 0; i > 8; i++)
	{
		num_ones += (shifted_byte & 0x01);
		shifted_byte = shifted_byte << 1;
	}

	/* I'm not fond of the ternary operator... */
	if(num_ones % 2)
	{
		return FALSE;
	}
	else
	{
		return TRUE;
	}
}

//http://stackoverflow.com/questions/2602823/in-c-c-whats-the-simplest-way-to-reverse-the-order-of-bits-in-a-byte
//
 // Reverse the order of bits within a byte.
 // Returns: The reversed byte value.
 //
unsigned char ReverseBits(unsigned char b) {
   b = (b & 0xF0) >> 4 | (b & 0x0F) << 4;
   b = (b & 0xCC) >> 2 | (b & 0x33) << 2;
   b = (b & 0xAA) >> 1 | (b & 0x55) << 1;
   return b;
}

int main()
{
	WDTCTL = WDTPW + WDTHOLD;  //Stop watchdog

	/* For now, assume PORT1 is device and PORT2 is host... they are both part of
	 * I/O port 1 on the MSP430. */
	P1DIR &= 0x00; //P1.0 is output to keyboard. P1.1 is input from keyboard.
	P1IFG &= ~PORT1_CLK;
	P1IES |= PORT1_CLK; //Interrupt Edge Select 0: LtoH 1:HtoL
	P1IE |= PORT1_CLK;

	BCSCTL1 &= ~(RSEL3 + RSEL2 + RSEL1 + RSEL0);
	BCSCTL1 |= (RSEL3); //2.3 MHz- try 1.6 Mhz...
	BCSCTL2 &= ~DIVS_3;
	BCSCTL2 &= DIVS_2; //Divide SubMain Clock by 4

	//BCSCTL3 &=	~LFXT1S_0; //Don't mess with other bits
   // BCSCTL3 |=	LFXT1S_2;
   		/* Set ACLK to Very Low Power Oscillator-
     	supposedly needs to be external, according to pg. 44 of device datasheet.
     	Frequency is 12000Hz approx. */
	TACCR0 = 0;
	TACTL = TASSEL_2 + ID_2 + MC_1; //Use SubMain Clock, divide by 4 again.
	TACCTL0 = CCIE;
	//TAIV = 0; /* Clear interrupts */
	//DCOCTL = DCO_
	//P1SEL |= CAPTURE_P2CLK; /* Use Timer Peripheral. */


	//keyIn_count = 0;
	//keyOut_count = 0;
	//goto LPM;
	P1In_buffer = 0x0000;
	IdleComm();

	BufferFlush();
	__bis_SR_register(GIE);

	SendBytetoATKeyboard(0xFF);
	Stall(65535);
	SendBytetoATKeyboard(0xEE);
	Stall(TIMER_FRQ/100);

	AT2XT_FSM();
}

ISR_BEGIN(PORT1_VECTOR)
void port1(void)
{
	#ifdef __DEBUG__
		counterc++;
	#endif

	static char keyOut_count = 0;
	static char keyIn_count = 0;

	if(HostMode)
	{
		//HostMode = FALSE;
		#ifdef __DEBUG__
			countera++;
		#endif
		if(P1IFG & PORT1_CLK)
		{
			//Bits 0 through Parity (second through tenth bit)
			if(keyOut_count <= 8)
			{
					//P1IFG &= ~PORT1_CLK; //Clear the flag which caused the interrupt!
					P1OUT &= ~PORT1_DATA;
					//P1OUT |= ((P1Out_buffer & 0x01) << (PORT1_DATA - 1));
					P1OUT |= ((P1Out_buffer & 0x01) << PORT1_DATA_PIN);
					P1Out_buffer = P1Out_buffer >> 1;
					keyOut_count++;
			}

			// 11
			else if(keyOut_count == 9)
			{
				IdleComm();
				keyOut_count++;
			}
			else if(keyOut_count == 10)
			{
				//test_byte = !(P1IN & PORT1_DATA);
				if(!(P1IN & PORT1_DATA))
				{
					DeviceACK = TRUE;
					keyOut_count = 0; /* In case of timeout, keyOut_count also needs to
										* be reset outside the ISR? */

					/* I don't think this works here... but I can't
					 * excatly recall the issues I had.. */
					//HostMode = FALSE;
				}
			}
		}
	}

	else
	{
			#ifdef __DEBUG__
				counterb++;
			#endif
			P1In_buffer = P1In_buffer << 1;
			P1In_buffer |= (P1IN & PORT1_DATA);
			keyIn_count++;

			if(keyIn_count >= 11)
			{
				P1IE &= ~PORT1_CLK;
				InhibitComm();

				//Prevent infinite recursion in case another byte is received.
				keyIn_count = 0;
				P1In_buffer = P1In_buffer >> PORT1_DATA_PIN;

				//If Start Bit is not 0 and Stop Bit is not 1... discard
				if((P1In_buffer & BITA) || !(P1In_buffer & BIT0))
				{
					BadKeys++;
					/* Do nothing */
				}
				else
				{
					P1In_buffer &= ~(BITA + BIT0);
					P1In_buffer = P1In_buffer >> 1; //Shift/remove stop bit
					//Check that parity is good...
					P1In_buffer = P1In_buffer >> 1;
					P1In_buffer = (short) ReverseBits((unsigned char) P1In_buffer);
					keycode_buffer[buffer_tail] = (char) P1In_buffer;
					buffer_tail = (++buffer_tail) % 16;
					// LPM3_EXIT;
				}
				P1In_buffer = 0x0000;
				/* Clear the buffer in to prevent the previous value from influencing the
				 * start bit/parity of the next value (because each bit is ORed with
				 * the current buffer- if any bits are 1 from bit reversal, they will
				 * STAY 1 while the next bytes are shifted in. */
				IdleComm();
				P1IE |= PORT1_CLK; //Keyboard must wait for line to be high for 50 milliseconds...
									//This is enough time to enable interrupts.
			}
			//P1IFG &= ~PORT1_CLK; //Clear the flag which caused the interrupt!
	}
	P1IFG &= ~PORT1_CLK; //Clear the flag which caused the interrupt!
}
ISR_END(port1)

ISR_BEGIN(TIMERA0_VECTOR)
void timera0(void)
{
	#ifdef __DEBUG__
		counterd++;
	#endif
	timeout = TRUE;
	StopTimer();
}
ISR_END(timera0)
