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
    UnmodifiedKey(u8),
    // InPause(u8), // Number of keycodes in pause left to handle- alternate impl.
    Inconsistent,
    ExpectingBufferClear,
}

pub struct Fsm {
    curr_state : state,
    expecting_pause : bool
}

impl Fsm {
    pub fn start() -> Fsm {
        Fsm { curr_state : state::NotInKey, expecting_pause : false }
    }

    pub fn run(&mut self, curr_reply : &ProcReply) -> Result<Cmd, Cmd> {
        let next_state = self.next_state(curr_reply).unwrap();

        let next_cmd = match &next_state {
            &state::NotInKey => { Ok(Cmd::WaitForKey) },
            &state::SimpleKey(k) => { Ok(Cmd::SendXTKey(keymap::to_xt(k))) },
            &state::PossibleBreakCode => { Ok(Cmd::WaitForKey) },
            &state::KnownBreakCode(b) => { Ok(Cmd::SendXTKey(keymap::to_xt(b) | 0x80)) },
            &state::UnmodifiedKey(u) => { Ok(Cmd::SendXTKey(u)) },
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

                    0xf0 => { Ok(state::PossibleBreakCode) },
                    0xe0 => { Ok(state::UnmodifiedKey(k)) },
                    // 0xe1 => { Ok(state::UnmodifiedKey) },

                    _ => { Ok(state::SimpleKey(k)) }
                }
            },
            (&state::SimpleKey(_), &ProcReply::SentKey(_)) => { Ok(state::NotInKey) },
            (&state::PossibleBreakCode, &ProcReply::GrabbedKey(k)) => {
                match k {
                    // LEDs => state::SetLED()
                    _ => { Ok(state::KnownBreakCode(k)) }
                }
            },
            (&state::KnownBreakCode(_), &ProcReply::SentKey(_)) => { Ok(state::NotInKey) },
            (&state::UnmodifiedKey(_), &ProcReply::SentKey(_)) => { Ok(state::NotInKey) },
            (&state::ExpectingBufferClear, &ProcReply::ClearedBuffer) => { Ok(state::NotInKey) },
            (_, _) => { Err(state::Inconsistent) },


            /* (NotInKey(_), NothingToDo) => { Ok(NotInKey) },
            (NotInKey(_), SentEchoExpectingEcho, */
            /* (_, _) => { Err(state::Inconsistent) } */
        }
    }
}
