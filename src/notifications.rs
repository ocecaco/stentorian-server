use errors::*;
use jsonrpc_core::{Notification, Params, Version};
use serde::Serialize;
use serde_json;
use stentorian::engine::{EngineEvent, MicrophoneState};
use stentorian::engine::Engine;

// impl<'a, 'c> GrammarNotification<'a, 'c> {
//     pub fn from_event(matcher: Option<&'a Matcher>, e: &'c GrammarEvent) -> Self {
//         match *e {
//             GrammarEvent::PhraseFinish(Some(Recognition {
//                 words: ref words_with_id,
//                 mut foreign,
//             })) => {
//                 let parse = if let Some(m) = matcher {
//                     if !foreign {
//                         m.perform_match(&words_with_id)
//                     } else {
//                         None
//                     }
//                 } else {
//                     foreign = true;
//                     None
//                 };

//                 let words_only = words_with_id
//                     .iter()
//                     .map(|&(ref w, _)| w as &str)
//                     .collect::<Vec<_>>();

//                 GrammarNotification::PhraseFinish {
//                     foreign_grammar: foreign,
//                     words: words_only,
//                     parse: parse,
//                 }
//             }
//             GrammarEvent::PhraseFinish(None) => {
//                 GrammarNotification::PhraseRecognitionFailure
//             }
//             GrammarEvent::PhraseStart => {
//                 GrammarNotification::PhraseStart
//             }
//         }
//     }
// }

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EngineNotification {
    Paused,
    MicrophoneStateChanged { state: MicrophoneState },
    UserChanged { name: Option<String> },
}

impl EngineNotification {
    pub fn from_event(engine: &Engine, e: EngineEvent) -> Result<Self> {
        match e {
            EngineEvent::Paused(cookie) => {
                engine.resume(cookie)?;

                let event = EngineNotification::Paused;

                Ok(event)
            }
            EngineEvent::MicrophoneState => {
                let state = engine.microphone_get_state()?;
                let event = EngineNotification::MicrophoneStateChanged { state };

                Ok(event)
            }
            EngineEvent::UserChanged => {
                let name = engine.get_current_user()?;
                let event = EngineNotification::UserChanged { name };

                Ok(event)
            }
        }
    }
}

pub fn create_notification<E>(id: u64, method: &str, event: &E) -> Result<String>
where
    E: Serialize,
{
    let v_event = serde_json::to_value(event)?;
    let v_id = serde_json::to_value(&id)?;
    let p = Params::Array(vec![v_id, v_event]);
    let n = Notification {
        jsonrpc: Some(Version::V2),
        method: method.to_owned(),
        params: Some(p),
    };

    Ok(serde_json::to_string(&n)?)
}
