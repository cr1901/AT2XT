use ::keymap;

/* #[derive(Debug)]
pub enum Leds {
    CapsLock,
    NumLock,
    ScrollLock,
} */

#[derive(Debug)]
pub enum Cmd {
    WaitForKey,
    ClearBuffer, // If Reset Occurs.
    ToggleLed(u8),
    SendXTKey(u8),
}

/* impl Cmd {
    pub fn init() -> Cmd {
        Cmd::WaitForKey
    }
} */


pub enum ProcReply {
    // JustInitialized,
    NothingToDo,
    GrabbedKey(u8),
    SentKey(u8),
    ClearedBuffer,
    LedToggled(u8),
    KeyboardReset,
    //SentEcho,
}

impl ProcReply {
    pub fn init() -> ProcReply {
        ProcReply::NothingToDo
    }
}

#[derive(Debug)]
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
    curr_state : State,
    expecting_pause : bool,
    led_mask : u8
}

impl Fsm {
    pub fn start() -> Fsm {
        Fsm { curr_state : State::NotInKey, expecting_pause : false, led_mask : 0 }
    }

    pub fn run(&mut self, curr_reply : &ProcReply) -> Result<Cmd, Cmd> {
        let next_state = self.next_state(curr_reply).unwrap();

        let next_cmd = match &next_state {
            &State::NotInKey => { Ok(Cmd::WaitForKey) },
            &State::SimpleKey(k) => { Ok(Cmd::SendXTKey(keymap::to_xt(k))) },
            &State::PossibleBreakCode => { Ok(Cmd::WaitForKey) },
            &State::KnownBreakCode(b) => { Ok(Cmd::SendXTKey(keymap::to_xt(b) | 0x80)) },
            &State::UnmodifiedKey(u) => { Ok(Cmd::SendXTKey(u)) },
            &State::ToggleLedFirst(l) => {
                match l {
                    0x7e => { Ok(Cmd::ToggleLed(self.led_mask ^ 0x01)) }, // Scroll
                    0x77 => { Ok(Cmd::ToggleLed(self.led_mask ^ 0x02)) }, // Num
                    0x58 => { Ok(Cmd::ToggleLed(self.led_mask ^ 0x04)) }, // Caps
                    _ => { Err(Cmd::WaitForKey) }
                }
            }
            &State::ExpectingBufferClear => { Ok(Cmd::ClearBuffer) }
            &State::Inconsistent => { Err(Cmd::WaitForKey) }
        };

        self.curr_state = next_state;
        next_cmd
    }

    fn next_state(&mut self, curr_reply : &ProcReply) -> Result<State, State> {
        match (&self.curr_state, curr_reply) {
            (_, &ProcReply::KeyboardReset) => { Ok(State::ExpectingBufferClear) },
            (&State::NotInKey, &ProcReply::NothingToDo) => { Ok(State::NotInKey) },
            (&State::NotInKey, &ProcReply::GrabbedKey(k)) => {
                match k {
                    0xaa => { Ok(State::NotInKey) },
                    // TODO: Actually, these should never be sent unprompted.
                    0xfa => { Ok(State::NotInKey) },
                    0xfe => { Ok(State::NotInKey) },
                    0xee => { Ok(State::NotInKey) },

                    0xf0 => { Ok(State::PossibleBreakCode) },
                    0xe0 => { Ok(State::UnmodifiedKey(k)) },
                    0xe1 => {
                        self.expecting_pause = true;
                        Ok(State::UnmodifiedKey(k))
                    },

                    _ => { Ok(State::SimpleKey(k)) }
                }
            },
            (&State::SimpleKey(_), &ProcReply::SentKey(_)) => { Ok(State::NotInKey) },
            (&State::PossibleBreakCode, &ProcReply::GrabbedKey(k)) => {
                match k {
                    // LEDs => State::ToggleLed()
                    0x7e => { Ok(State::ToggleLedFirst(k)) },
                    0x77 => { if self.expecting_pause {
                                self.expecting_pause = false;
                                Ok(State::KnownBreakCode(k))
                            } else {
                                Ok(State::ToggleLedFirst(k))
                            }
                    },
                    0x58 => { Ok(State::ToggleLedFirst(k)) },
                    _ => { Ok(State::KnownBreakCode(k)) }
                }
            },
            (&State::KnownBreakCode(_), &ProcReply::SentKey(_)) => { Ok(State::NotInKey) },
            (&State::UnmodifiedKey(_), &ProcReply::SentKey(_)) => { Ok(State::NotInKey) },
            (&State::ToggleLedFirst(l), &ProcReply::LedToggled(m)) => {
                self.led_mask = m;
                Ok(State::KnownBreakCode(l))
            },
            (&State::ExpectingBufferClear, &ProcReply::ClearedBuffer) => { Ok(State::NotInKey) },
            (_, _) => { Err(State::Inconsistent) },


            /* (NotInKey(_), NothingToDo) => { Ok(NotInKey) },
            (NotInKey(_), SentEchoExpectingEcho, */
            /* (_, _) => { Err(State::Inconsistent) } */
        }
    }
}
