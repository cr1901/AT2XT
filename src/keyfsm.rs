use ::keymap;

#[derive(Debug)]
pub enum Cmd {
    WaitForKey,
    ClearBuffer, // If Reset Occurs.
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
    KeyboardReset,
    //SentEcho,
}

impl ProcReply {
    pub fn init() -> ProcReply {
        ProcReply::NothingToDo
    }
}

#[derive(Debug)]
enum state {
    NotInKey,
    SimpleKey(u8),
    PossibleBreakCode,
    KnownBreakCode(u8),
    //UnmodifiedKey,
    Inconsistent,
    ExpectingBufferClear,
    //ExpectingEcho,
    //SendXTByte(u8),
    //GetXTByteF0,
    //InPause(u8),  // Number of keycodes in pause left to handle.
}

pub struct Fsm {
    curr_state : state,
}

impl Fsm {
    pub fn start() -> Fsm {
        Fsm { curr_state : state::NotInKey }
    }

    pub fn run(&mut self, curr_reply : &ProcReply) -> Result<Cmd, Cmd> {
        let next_state = self.next_state(curr_reply).unwrap();

        let next_cmd = match &next_state {
            &state::NotInKey => { Ok(Cmd::WaitForKey) },
            &state::SimpleKey(k) => { Ok(Cmd::SendXTKey(keymap::to_xt(k))) },
            &state::PossibleBreakCode => { Ok(Cmd::WaitForKey) },
            &state::KnownBreakCode(b) => { Ok(Cmd::SendXTKey(keymap::to_xt(b) | 0x80)) },
            &state::ExpectingBufferClear => { Ok(Cmd::ClearBuffer) }
            &state::Inconsistent => { Err(Cmd::WaitForKey) }
        };

        self.curr_state = next_state;
        next_cmd
    }

    fn next_state(&self, curr_reply : &ProcReply) -> Result<state, state> {
        match (&self.curr_state, curr_reply) {
            (_, &ProcReply::KeyboardReset) => { Ok(state::ExpectingBufferClear) },
            (&state::NotInKey, &ProcReply::NothingToDo) => { Ok(state::NotInKey) },
            (&state::NotInKey, &ProcReply::GrabbedKey(k)) => {
                match k {
                    0xaa => { Ok(state::NotInKey) },
                    // TODO: Actually, these should never be sent unprompted.
                    0xfa => { Ok(state::NotInKey) },
                    0xfe => { Ok(state::NotInKey) },
                    0xee => { Ok(state::NotInKey) },

                    0xf0 => {
                        panic!(); // This doesn't!
                        Ok(state::PossibleBreakCode)
                    },

                    //0xe0 => { Ok(state::UnmodifiedKey) },
                    //0xe1 => { Ok(state::UnmodifiedKey) },

                    _ => {
                        // panic!(); // This panics!
                        Ok(state::SimpleKey(k)) }
                }
            },
            (&state::SimpleKey(_), &ProcReply::SentKey(_)) => { Ok(state::NotInKey) },
            (&state::PossibleBreakCode, &ProcReply::GrabbedKey(k)) => {
                match k {
                    // LEDs => state::SetLED()
                    _ => { Ok(state::KnownBreakCode(k)) }
                }
            },


            (&state::ExpectingBufferClear, &ProcReply::ClearedBuffer) => { Ok(state::NotInKey) },
            (_, _) => { Err(state::Inconsistent) },


            /* (NotInKey(_), NothingToDo) => { Ok(NotInKey) },
            (NotInKey(_), SentEchoExpectingEcho, */
            /* (_, _) => { Err(state::Inconsistent) } */
        }
    }
}
