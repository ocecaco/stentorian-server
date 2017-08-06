use jsonrpc_core::{Notification, Version, Params};
use serde_json;
use serde::Serialize;
use stentorian::resultparser::{Matcher, Match};
use stentorian::engine::{GrammarEvent, Recognition, EngineEvent,
                         Attribute, MicrophoneState};
use stentorian::engine::Engine;
use errors::*;

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(unknown_lints, enum_variant_names)]
pub enum GrammarNotification<'a, 'c> {
    PhraseFinish {
        foreign_grammar: bool,
        words: Vec<&'c str>,
        parse: Option<Match<'a>>,
    },
    PhraseRecognitionFailure,
    PhraseStart,
}

impl<'a, 'c> GrammarNotification<'a, 'c> {
    pub fn from_event(matcher: &'a Matcher, e: &'c GrammarEvent) -> Self {
        match *e {
            GrammarEvent::PhraseFinish(Some(Recognition {
                words: ref words_with_id,
                foreign,
            })) => {
                let parse = if !foreign {
                    matcher.perform_match(&words_with_id)
                } else {
                    None
                };

                let words_only = words_with_id
                    .iter()
                    .map(|&(ref w, _)| w as &str)
                    .collect::<Vec<_>>();

                GrammarNotification::PhraseFinish {
                    foreign_grammar: foreign,
                    words: words_only,
                    parse: parse,
                }
            }
            GrammarEvent::PhraseFinish(None) => {
                GrammarNotification::PhraseRecognitionFailure
            }
            GrammarEvent::PhraseStart => {
                GrammarNotification::PhraseStart
            }
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EngineNotification {
    Paused,
    MicrophoneStateChanged { state: MicrophoneState },
}

impl EngineNotification {
    pub fn from_event(engine: &Engine, e: EngineEvent) -> Result<Self> {
        match e {
            EngineEvent::Paused(cookie) => {
                engine.resume(cookie)?;

                let event = EngineNotification::Paused;

                Ok(event)
            }
            EngineEvent::AttributeChanged(a) => {
                let event = match a {
                    Attribute::MicrophoneState => {
                        let state = engine.microphone_get_state()?;
                        EngineNotification::MicrophoneStateChanged { state }
                    }
                };

                Ok(event)
            }
        }
    }
}

pub fn create_notification<E>(id: u64, method: &str, event: &E) -> Result<String>
    where E: Serialize
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
