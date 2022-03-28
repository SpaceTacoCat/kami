use crate::app_state::SharedState;
use crate::buffer::EventHandlerOutcome;
use crate::KamiEvent;
use tokio::sync::mpsc::Receiver;
use winit::event::ModifiersState;
use winit::event_loop::EventLoopProxy;

#[derive(Debug)]
pub enum StateEvent {
    ModifiersChange(ModifiersState),
    CharInput(char),
}

pub async fn state_loop(
    proxy: EventLoopProxy<KamiEvent>,
    mut state_rx: Receiver<StateEvent>,
    app_state: SharedState,
) {
    while let Some(event) = state_rx.recv().await {
        match event {
            StateEvent::ModifiersChange(_ms) => {
                // TODO: handle keyboard control
            }
            // All input, translated to unicode, including backspace and delete comes here
            StateEvent::CharInput(c) => match app_state.read().await.handle_character(c).await {
                EventHandlerOutcome::Redraw => {
                    proxy.send_event(KamiEvent::RequestRedraw).unwrap();
                }
                EventHandlerOutcome::None => {}
            },
        }
    }
}
