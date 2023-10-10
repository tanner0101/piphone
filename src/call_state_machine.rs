pub enum CallState {
    Idle,
    OutgoingCall,
    IncomingCall,
    InProgressCall,
}

pub enum CallSwitchState {
    NoEdge,
    Pressed,
    Released,
}

pub struct Call {
    state: CallState,
}

pub enum PacketType {
    Ring,
    RingAck,
    VoiceData,
}

impl Call {
    fn new() -> Self {
        Call {
            state: CallState::Idle,
        }
    }

    fn dispatch(mut self, header: Option<PacketType>, call_switch: CallSwitchState) {
        use CallState::*;
        use CallSwitchState::*;
        use PacketType::*;
        let mut outgoing_call_time = 0;
        self.state = match (self.state, header, call_switch) {
            (Idle, None, NoEdge) => CallState::Idle,
            (OutgoingCall | InProgressCall, _, Pressed) => Idle,

            (Idle, Some(Ring), NoEdge | Pressed) => IncomingCall,

            (OutgoingCall, Some(RingAck) | None, NoEdge) => {
                outgoing_call_time += 1;

                match outgoing_call_time > 10 {
                    false => OutgoingCall,
                    true => Idle,
                }
            }
            (Idle, None, Released) => OutgoingCall,

            (Idle, Some(Ring), Released) => InProgressCall,
            (IncomingCall, Some(Ring) | None, Released) => InProgressCall,
            // (Idle, None, CallSwitchState::Pressed) => CallState::Idle,
        }
    }
}
