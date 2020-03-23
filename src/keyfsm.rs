use bitflags::bitflags;

mod keymap {
    static KEYCODE_LUT : [u8; 132] =
    // 0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F
    [0x00,0x43,0x00,0x3F,0x3D,0x3B,0x3C,0x58,0x00,0x44,0x42,0x40,0x3E,0x0F,0x29,0x00,
    0x00,0x38,0x2A,0x00,0x1D,0x10,0x02,0x00,0x00,0x00,0x2C,0x1F,0x1E,0x11,0x03,0x00,
    0x00,0x2E,0x2D,0x20,0x12,0x05,0x04,0x00,0x00,0x39,0x2F,0x21,0x14,0x13,0x06,0x00,
    0x00,0x31,0x30,0x23,0x22,0x15,0x07,0x00,0x00,0x00,0x32,0x24,0x16,0x08,0x09,0x00,
    0x00,0x33,0x25,0x17,0x18,0x0B,0x0A,0x00,0x00,0x34,0x35,0x26,0x27,0x19,0x0C,0x00,
    0x00,0x00,0x28,0x00,0x1A,0x0D,0x00,0x00,0x3A,0x36,0x1C,0x1B,0x00,0x2B,0x00,0x00,
    0x00,0x00,0x00,0x00,0x00,0x00,0x0E,0x00,0x00,0x4F,0x00,0x4B,0x47,0x00,0x00,0x00,
    0x52,0x53,0x50,0x4C,0x4D,0x48,0x01,0x45,0x57,0x4E,0x51,0x4A,0x37,0x49,0x46,0x00,
    0x00,0x00,0x00,0x41];

    pub fn to_xt(at_in: u8) -> Option<u8> {
        KEYCODE_LUT.get(usize::from(at_in)).copied()
    }
}

pub enum Cmd {
    WaitForKey,
    ClearBuffer, // If Reset Occurs.
    ToggleLed(LedMask),
    SendXTKey(u8),
}

impl Cmd {
    // XT command
    pub const SELF_TEST_PASSED: u8 = 0xaa;

    // AT commands
    pub const SET_LEDS: u8 = 0xed;
    #[allow(dead_code)]
    pub const ECHO: u8 = 0xee;
    pub const RESET: u8 = 0xff;
}

bitflags! {
    #[derive(Default)]
    pub struct LedMask: u8 {
        const SCROLL = 0b0000_0001;
        const NUM = 0b0000_0010;
        const CAPS = 0b0000_0100;
    }
}

pub enum ProcReply {
    // JustInitialized,
    NothingToDo,
    GrabbedKey(u8),
    SentKey(u8),
    ClearedBuffer,
    LedToggled(LedMask),
    KeyboardReset,
    //SentEcho,
}

impl ProcReply {
    pub fn init() -> ProcReply {
        ProcReply::NothingToDo
    }
}

enum State {
    NotInKey,
    SimpleKey(u8),
    PossibleBreakCode,
    KnownBreakCode(u8),
    UnmodifiedKey(u8),
    ToggleLedFirst(u8),
    // InPause(u8), // Number of keycodes in pause left to handle- alternate impl.
    Inconsistent,
    ExpectingBufferClear,
}

pub struct Fsm {
    curr_state: State,
    expecting_pause: bool,
    led_mask: LedMask,
}

impl Fsm {
    #[allow(dead_code)]
    const ERROR1: u8 = 0x00;
    const CAPS: u8 = 0x58;
    const NUM: u8 = 0x77;
    const SCROLL: u8 = 0x7e;
    const SELF_TEST_PASSED: u8 = 0xaa;
    const PREFIX: u8 = 0xe0;
    const PREFIX_PAUSE: u8 = 0xe1;
    const ECHO: u8 = 0xee;
    const BREAK: u8 = 0xf0;
    const ACK: u8 = 0xfa;
    #[allow(dead_code)]
    const SELF_TEST_FAILED1: u8 = 0xfc;
    #[allow(dead_code)]
    const SELF_TEST_FAILED2: u8 = 0xfd;
    const NAK: u8 = 0xfe;
    #[allow(dead_code)]
    const ERROR2: u8 = 0xff;

    pub fn start() -> Fsm {
        Fsm {
            curr_state: State::NotInKey,
            expecting_pause: false,
            led_mask: Default::default(),
        }
    }

    pub fn run(&mut self, curr_reply: &ProcReply) -> Result<Cmd, ()> {
        let next_state = self.next_state(curr_reply);

        let next_cmd = match next_state {
            State::NotInKey | State::PossibleBreakCode => Ok(Cmd::WaitForKey),
            State::SimpleKey(k) => match keymap::to_xt(k) {
                Some(k) => Ok(Cmd::SendXTKey(k)),
                None => Err(()),
            },
            State::KnownBreakCode(b) => match keymap::to_xt(b) {
                Some(b) => Ok(Cmd::SendXTKey(b | 0x80)),
                None => Err(()),
            },
            State::UnmodifiedKey(u) => Ok(Cmd::SendXTKey(u)),
            State::ToggleLedFirst(l) => match l {
                Self::SCROLL => Ok(Cmd::ToggleLed(self.led_mask ^ LedMask::SCROLL)),
                Self::NUM => Ok(Cmd::ToggleLed(self.led_mask ^ LedMask::NUM)),
                Self::CAPS => Ok(Cmd::ToggleLed(self.led_mask ^ LedMask::CAPS)),
                _ => Err(()),
            },
            State::ExpectingBufferClear => Ok(Cmd::ClearBuffer),
            State::Inconsistent => Err(()),
        };

        self.curr_state = next_state;
        next_cmd
    }

    fn next_state(&mut self, curr_reply: &ProcReply) -> State {
        match (&self.curr_state, curr_reply) {
            (_, &ProcReply::KeyboardReset) => State::ExpectingBufferClear,
            (&State::NotInKey, &ProcReply::NothingToDo)
            | (&State::SimpleKey(_), &ProcReply::SentKey(_))
            | (&State::KnownBreakCode(_), &ProcReply::SentKey(_))
            | (&State::UnmodifiedKey(_), &ProcReply::SentKey(_))
            | (&State::ExpectingBufferClear, &ProcReply::ClearedBuffer) => State::NotInKey,
            (&State::NotInKey, &ProcReply::GrabbedKey(k)) => {
                match k {
                    // TODO: 0xfa, 0xfe, and 0xee should never be sent unprompted.
                    Self::SELF_TEST_PASSED | Self::ACK | Self::NAK | Self::ECHO => State::NotInKey,
                    Self::BREAK => State::PossibleBreakCode,
                    Self::PREFIX | Self::PREFIX_PAUSE => {
                        self.expecting_pause = (k == Self::PREFIX_PAUSE);
                        State::UnmodifiedKey(k)
                    },

                    _ => State::SimpleKey(k),
                }
            }
            (&State::PossibleBreakCode, &ProcReply::GrabbedKey(k)) => {
                match k {
                    Self::SCROLL | Self::CAPS => State::ToggleLedFirst(k),
                    Self::NUM => {
                        if self.expecting_pause {
                            self.expecting_pause = false;
                            State::KnownBreakCode(k)
                        } else {
                            State::ToggleLedFirst(k)
                        }
                    }
                    _ => State::KnownBreakCode(k),
                }
            }
            (&State::ToggleLedFirst(l), &ProcReply::LedToggled(m)) => {
                self.led_mask = m;
                State::KnownBreakCode(l)
            }
            (_, _) => State::Inconsistent,
        }
    }
}
