use crate::net_util::PacketType;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CallState {
    Idle,
    OutgoingCall,
    IncomingCall,
    InProgressCall,
}

#[derive(Debug)]
pub enum CallSwitchEdge {
    NoEdge,
    Active,
    Inactive,
}

#[derive(Clone)]
pub enum CallSwitchState {
    Active,
    Inactive,
}

pub struct CallSwitch {
    current_state: Option<CallSwitchState>,
}

impl CallSwitch {
    pub fn new() -> Self {
        CallSwitch {
            current_state: None,
        }
    }

    pub fn dispatch(&mut self, new_switch_state: &CallSwitchState) -> CallSwitchEdge {
        use CallSwitchState::*;

        let edge: CallSwitchEdge = match (self.current_state.as_ref(), new_switch_state) {
            (None, Active) => CallSwitchEdge::Active,
            (None, Inactive) => CallSwitchEdge::Inactive,
            (Some(Inactive), Active) => CallSwitchEdge::Active,
            (Some(Active), Inactive) => CallSwitchEdge::Inactive,
            (Some(Inactive), Inactive) => CallSwitchEdge::NoEdge,
            (Some(Active), Active) => CallSwitchEdge::NoEdge,
        };

        self.current_state = Some(new_switch_state.clone());
        return edge;
    }
}

pub struct Call {
    pub state: CallState,
}

impl Call {
    pub fn new() -> Self {
        Call {
            state: CallState::Idle,
        }
    }

    pub fn dispatch(
        &mut self,
        packet_type: &Option<PacketType>,
        call_switch_edge: &CallSwitchEdge,
    ) -> &mut Call {
        use CallState::*;
        use CallSwitchEdge::*;
        use PacketType::*;
        let new_state = match (self.state.clone(), packet_type, call_switch_edge) {
            /* First we walk through nominal path. */

            /* Generally nothing happening */
            (Idle, None, NoEdge) => CallState::Idle,

            /* Switch activated, call other phone. */
            (Idle, None, Active) => OutgoingCall,

            /* Calling other phone - TODO implement timeout eventually */
            (OutgoingCall, Some(RingAck) | None, NoEdge) => OutgoingCall,

            /* Once voice data is received, call has started. */
            (OutgoingCall, Some(VoiceData), NoEdge) => InProgressCall,

            /* Start getting called by the other phone. */
            (Idle, Some(Ring), NoEdge) => IncomingCall,

            /* Still being rung - TODO add timeouts */
            (IncomingCall, Some(Ring) | None, NoEdge) => IncomingCall,

            /* Answering incoming call. */
            (IncomingCall, Some(Ring) | None, Active) => InProgressCall,

            /* Ongoing call - TODO implement timeout for None */
            (InProgressCall, Some(VoiceData) | None, NoEdge) => InProgressCall,

            /* Hangup */
            (OutgoingCall | InProgressCall, _, Inactive) => Idle,
            
            /* Program likely restarted during a call. */
            (Idle, Some(VoiceData), Active) => InProgressCall,

            /* Somehow we called each other at the same time, just answer. */
            (OutgoingCall, Some(Ring), NoEdge) => InProgressCall,
            (Idle, Some(Ring), Active) => InProgressCall,

            /* These cases are nominal edge cases (when moving from states). */
            (Idle, Some(RingAck), _) => Idle,
            (Idle, Some(VoiceData), _) => Idle,
            (InProgressCall, Some(Ring), NoEdge) => InProgressCall,

            /* These states shouldn't happen. Error and reset. */
            (Idle, None | Some(Ring), Inactive)
            | (IncomingCall, Some(Ring), Inactive)
            | (IncomingCall, Some(RingAck) | Some(VoiceData), _)
            | (IncomingCall, None, Inactive)
            | (OutgoingCall, _, Active)
            | (InProgressCall, Some(RingAck), NoEdge)
            | (InProgressCall, _, Active) => {
                println!(
                    "\nERROR: Call state machine: \n\
                    CallState: {:?}\n\
                    Packet Type: {:?}\n\
                    Call Switch: {:?}\n",
                    self.state, packet_type, call_switch_edge
                );

                Idle
            }
        };

        if new_state != self.state {
            println!("call state changed: {:?}", new_state);
        }
        self.state = new_state;
        return self;
    }
}
