#include <msp430g2211.h>
#include "AT2XT.h"

//Debug variables
#ifdef __DEBUG__
	short AA_count = 0;
#endif	
	
static char led_code = 0;

static void store_keycode();
static unsigned char process_keycode();
//static unsigned char convert_keycode(unsigned char keycode);
static void send_new_keycode(unsigned char new_keycode);

static const unsigned char ATXT_Table[132]=
//0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F
{0x00,0x43,0x00,0x3F,0x3D,0x3B,0x3C,0x58,0x00,0x44,0x42,0x40,0x3E,0x0F,0x29,0x00,
0x00,0x38,0x2A,0x00,0x1D,0x10,0x02,0x00,0x00,0x00,0x2C,0x1F,0x1E,0x11,0x03,0x00,
0x00,0x2E,0x2D,0x20,0x12,0x05,0x04,0x00,0x00,0x39,0x2F,0x21,0x14,0x13,0x06,0x00,
0x00,0x31,0x30,0x23,0x22,0x15,0x07,0x00,0x00,0x00,0x32,0x24,0x16,0x08,0x09,0x00,
0x00,0x33,0x25,0x17,0x18,0x0B,0x0A,0x00,0x00,0x34,0x35,0x26,0x27,0x19,0x0C,0x00,
0x00,0x00,0x28,0x00,0x1A,0x0D,0x00,0x00,0x3A,0x36,0x1C,0x1B,0x00,0x2B,0x00,0x00,
0x00,0x00,0x00,0x00,0x00,0x00,0x0E,0x00,0x00,0x4F,0x00,0x4B,0x47,0x00,0x00,0x00,
0x52,0x53,0x50,0x4C,0x4D,0x48,0x01,0x45,0x57,0x4E,0x51,0x4A,0x37,0x49,0x46,0x00,
0x00,0x00,0x00,0x41}; //Entry 0x84 is 0x54
	

typedef enum AT2XT_fsm_states
{
	CLEAR,
	
	CHECK_CHAR_NEW,
	GET_XT_BYTE_NEW,
	CHECK_BUFFER_NEW,
	
	CHECK_BUFFER_F0,
	WAIT_FOR_NEXT_BYTE_F0,
	GET_XT_BYTE_F0,
	
	SEND_UNMODIFIED_KEYCODE,
	HANDLE_COMMAND_BYTE,
	
	DONE_FSM
}AT2XT_FSM_STATES;

AT2XT_FSM_STATES curr_state = CLEAR;

typedef enum pause_states
{
	OUTSIDE_PAUSE,
	IN_PAUSE
}PAUSE_STATES;


//FSM to-do:
//Implement command-response handling
//Implement bad-key handling
//Implement PAUSE logic to stop NUM LOCK interference
void AT2XT_FSM()
{
	//char FSM_sim = TRUE;
	
	//Arrow keys beep for some reason on Windows keyboard...
	//Checkit PC vs AT vs 101 keyboard test (Pause mapped to Numlock- 101/AT, RCTRL mapped to backspace- AT)
	
	//Make static? Makes any difference?
	unsigned char curr_keycode;
	PAUSE_STATES curr_pause_state = OUTSIDE_PAUSE;
	//printf("AT Code  XT Code\n");
	while(TRUE)
	{
		switch(curr_state)
		{
		case CLEAR:
			store_keycode();
			curr_state = CHECK_CHAR_NEW;
			break;
			
		case CHECK_CHAR_NEW:
			curr_keycode = process_keycode();
			switch(curr_keycode)
			{
			/* case 0x00:
				curr_state = DONE_FSM;
				break; */
			// Ignore certain keycodes.	
			case 0xFA:
			case 0xAA:
			case 0xFE: //Check for this code! Also known bugs: LOCK LEDs can go out of sync during Checkit.
			case 0xEE:
				curr_state = CHECK_BUFFER_NEW;
				break;
					
			case 0xF0:
				curr_state = CHECK_BUFFER_F0;
				break;
				
			case 0xE0:
			case 0xE1:
				curr_state = SEND_UNMODIFIED_KEYCODE;
				break;
				
			default:
				curr_state = GET_XT_BYTE_NEW;
				break;
			}
			break;
			
		case GET_XT_BYTE_NEW:
			send_new_keycode(ATXT_Table[curr_keycode]);
			curr_state = CHECK_BUFFER_NEW;
			break;
			
		case CHECK_BUFFER_NEW:
			if((buffer_tail - buffer_head) == 0)
			{
				curr_state = CLEAR;
			}
			else
			{
				curr_state = CHECK_CHAR_NEW;
			}
			break;
			
		case CHECK_BUFFER_F0:
			if((buffer_tail - buffer_head) == 0)
			{
				curr_state = WAIT_FOR_NEXT_BYTE_F0;
			}
			else
			{
				curr_state = GET_XT_BYTE_F0;
			}
			break;
		
		case WAIT_FOR_NEXT_BYTE_F0:
			store_keycode();
			curr_state = GET_XT_BYTE_F0;
			break;
		
		case GET_XT_BYTE_F0:
			//Can be done all at once
			curr_keycode = process_keycode();
			
			switch(curr_keycode)
			{
				case 0x7E:
					SendBytetoATKeyboard(0xED);
					led_code ^= 0x01;
					Stall(TIMER_FRQ/100);
					SendBytetoATKeyboard(led_code); //Scroll lock off
					break;
					
				case 0x77:	
					/* The pause/break keycode embeds
					 * the numlock make/break code... LEDs go
					 * out of sync! */
					switch(curr_pause_state)
					{
						//If we got here (0xF0 0x77), PAUSE is over!
						case IN_PAUSE:
							curr_pause_state = OUTSIDE_PAUSE;
							break;
						default:
							SendBytetoATKeyboard(0xED);
							led_code ^= 0x02;
							Stall(TIMER_FRQ/100);
							SendBytetoATKeyboard(led_code); //Num lock off
							break;
					}
					break;
					
				case 0x58:
					SendBytetoATKeyboard(0xED);
					led_code ^= 0x04;
					Stall(TIMER_FRQ/100);
					SendBytetoATKeyboard(led_code); //Caps lock off
					break;
					
				default:
					break;
			}
			send_new_keycode(ATXT_Table[curr_keycode] | 0x80);
			curr_state = CHECK_BUFFER_NEW;
			break;
			
		case SEND_UNMODIFIED_KEYCODE:
			//0xE0 and 0xE1 are handled as special cases
			if(curr_keycode == 0xE1)
			{
				curr_pause_state = IN_PAUSE;
			}
				
			send_new_keycode(curr_keycode);
			curr_state = CHECK_BUFFER_NEW;
			break;
			
		/* case DONE_FSM:
			FSM_sim = 0;
			break; */
		}
	}
	//return EXIT_SUCCESS;
}

void store_keycode()
{
	/* Go to sleep, wait for Interrupt */
	//SendBytetoATKeyboard(0xFF);
	do
	{
		//__bis_SR_register(LPM3_bits + GIE);
		//SendBytetoATKeyboard(0xF2);
		//SendBytetoATKeyboard(0xFF);
		//SendBytetoATKeyboard(0xFF);
		//SendBytetoATKeyboard(0xFF);
		
		if((INPUT_REG & CAPTURE_P2CLK) == 0)
		{
			StartTimer(TIMER_FRQ/50); /* 1/50th of a second = 20ms */
			while((INPUT_REG & CAPTURE_P2CLK) == 0)
			{
				/* If more than 20 ms pass since the timer started, 
 				* it's time to reset. */
				if(timeout) /* Timer has stopped itself if timeout */
				{
					/* On soft reset, sending two reset commands in a 
					 * short period of time may cause the keyboard to ask for the command
					 * to be resent (0xFE). This is okay/does not affect operation. */ 
					SendBytetoATKeyboard(0xFF);
					SendBytetoPC(0xAA);
					buffer_flush();
					led_code = 0x00;
					#ifdef __DEBUG__
						AA_count++;
					#endif
					//PAUSE(500000);
					break;
				}
			}
			/* Might not be necessary... */
			StopTimer();
		}
		
	}while((buffer_head - buffer_tail) == 0);
}

unsigned char process_keycode()
{
	unsigned char keycode;
	/* Hex is unsigned by default?:
	http://stackoverflow.com/questions/9640359/scanf-data-in-hex-format-into-unsigned-shorts
	*/
	keycode = keycode_buffer[buffer_head];
	//printf("%hX,", keycode);
	buffer_head = (++buffer_head)%16;
	return keycode;	
}

void send_new_keycode(unsigned char new_keycode)
{
	SendBytetoPC(new_keycode);
}
