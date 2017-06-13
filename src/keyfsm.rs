pub enum cmd {

}

enum proc_reply {
    NothingToDo,
    SentEcho,
}

enum state {
    ClearBuffer,
    Inconsistent,
    ExpectingEcho,
    //SendXTByte(u8),
    //GetXTByteF0,
    //InPause(u8),  // Number of keycodes in pause left to handle.
}

pub struct Fsm {
    curr_state : state,
}

impl state {
    pub fn start() -> fsm {
        Fsm { curr_state : ClearBuffer }
    }

    pub fn run(&mut self, curr_reply : proc_reply) -> Result<cmd> {

    }

    fn next_state(&self, curr_reply : proc_reply) -> Result<state> {
        match (self.curr_state, curr_reply) {
            (ClearBuffer(_), NothingToDo) => { Ok(ClearBuffer) },
            (ClearBuffer(_), SentEchoExpectingEcho,
            (_, _) => { Err(Inconsistent) }
        }
    }
}
