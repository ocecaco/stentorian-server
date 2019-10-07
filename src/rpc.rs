use crate::errors::MyError as Error;
use jsonrpc_core;
use jsonrpc_derive::rpc;
use stentorian::engine::MicrophoneState;
use stentorian::grammar::Grammar;

#[rpc(server)]
pub trait RpcCommand {
    #[rpc(name = "command_grammar_load")]
    fn load(&self, grammar: Grammar) -> Result<u64, Error>;

    #[rpc(name = "command_grammar_unload")]
    fn unload(&self, grammar_id: u64) -> Result<(), Error>;

    #[rpc(name = "command_grammar_rule_activate")]
    fn rule_activate(&self, grammar_id: u64, rule_name: String) -> Result<(), Error>;

    #[rpc(name = "command_grammar_rule_deactivate")]
    fn rule_deactivate(&self, grammar_id: u64, rule_name: String) -> Result<(), Error>;

    #[rpc(name = "command_grammar_list_append")]
    fn list_append(&self, grammar_id: u64, list_name: String, word: String) -> Result<(), Error>;

    #[rpc(name = "command_grammar_list_remove")]
    fn list_remove(&self, grammar_id: u64, list_name: String, word: String) -> Result<(), Error>;

    #[rpc(name = "command_grammar_list_clear")]
    fn list_clear(&self, grammar_id: u64, list_name: String) -> Result<(), Error>;
}

#[rpc(server)]
pub trait RpcSelect {
    #[rpc(name = "select_grammar_load")]
    fn load(&self, select_words: Vec<String>, through_words: Vec<String>) -> Result<u64, Error>;

    #[rpc(name = "select_grammar_unload")]
    fn unload(&self, grammar_id: u64) -> Result<(), Error>;

    #[rpc(name = "select_grammar_activate")]
    fn activate(&self, grammar_id: u64) -> Result<(), Error>;

    #[rpc(name = "select_grammar_deactivate")]
    fn deactivate(&self, grammar_id: u64) -> Result<(), Error>;

    #[rpc(name = "select_grammar_text_set")]
    fn text_set(&self, grammar_id: u64, text: String) -> Result<(), Error>;

    #[rpc(name = "select_grammar_text_change")]
    fn text_change(&self, grammar_id: u64, start: u32, end: u32, text: String)
        -> Result<(), Error>;

    #[rpc(name = "select_grammar_text_delete")]
    fn text_delete(&self, grammar_id: u64, start: u32, end: u32) -> Result<(), Error>;

    #[rpc(name = "select_grammar_text_insert")]
    fn text_insert(&self, grammar_id: u64, start: u32, text: String) -> Result<(), Error>;

    #[rpc(name = "select_grammar_text_get")]
    fn text_get(&self, grammar_id: u64) -> Result<String, Error>;
}

#[rpc(server)]
pub trait RpcDictation {
    #[rpc(name = "dictation_grammar_load")]
    fn load(&self) -> Result<u64, Error>;

    #[rpc(name = "dictation_grammar_unload")]
    fn unload(&self, grammar_id: u64) -> Result<(), Error>;

    #[rpc(name = "dictation_grammar_activate")]
    fn activate(&self, grammar_id: u64) -> Result<(), Error>;

    #[rpc(name = "dictation_grammar_deactivate")]
    fn deactivate(&self, grammar_id: u64) -> Result<(), Error>;

    #[rpc(name = "dictation_grammar_context_set")]
    fn context_set(&self, grammar_id: u64, context: String) -> Result<(), Error>;
}

#[rpc(server)]
pub trait RpcCatchall {
    #[rpc(name = "catchall_grammar_load")]
    fn load(&self) -> Result<u64, Error>;

    #[rpc(name = "catchall_grammar_unload")]
    fn unload(&self, rammar_id: u64) -> Result<(), Error>;

    #[rpc(name = "catchall_grammar_activate")]
    fn activate(&self, grammar_id: u64) -> Result<(), Error>;

    #[rpc(name = "catchall_grammar_deactivate")]
    fn deactivate(&self, grammar_id: u64) -> Result<(), Error>;
}

#[rpc(server)]
pub trait RpcEngine {
    #[rpc(name = "engine_register")]
    fn register(&self) -> Result<u64, Error>;

    #[rpc(name = "engine_unregister")]
    fn unregister(&self, grammar_id: u64) -> Result<(), Error>;

    #[rpc(name = "microphone_set_state")]
    fn microphone_set_state(&self, state: MicrophoneState) -> Result<(), Error>;

    #[rpc(name = "microphone_get_state")]
    fn microphone_get_state(&self) -> Result<MicrophoneState, Error>;

    #[rpc(name = "get_current_user")]
    fn get_current_user(&self) -> Result<Option<String>, Error>;
}
