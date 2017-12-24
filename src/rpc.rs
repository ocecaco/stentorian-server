use errors::Error;
use stentorian::engine::MicrophoneState;
use stentorian::grammar::Grammar;

build_rpc_trait! {
    pub trait RpcCommand {
        #[rpc(name = "command_grammar_load")]
        fn load(&self, Grammar) -> Result<u64, Error>;

        #[rpc(name = "command_grammar_unload")]
        fn unload(&self, u64) -> Result<(), Error>;

        #[rpc(name = "command_grammar_rule_activate")]
        fn rule_activate(&self, u64, String) -> Result<(), Error>;

        #[rpc(name = "command_grammar_rule_deactivate")]
        fn rule_deactivate(&self, u64, String) -> Result<(), Error>;

        #[rpc(name = "command_grammar_list_append")]
        fn list_append(&self, u64, String, String) -> Result<(), Error>;

        #[rpc(name = "command_grammar_list_remove")]
        fn list_remove(&self, u64, String, String) -> Result<(), Error>;

        #[rpc(name = "command_grammar_list_clear")]
        fn list_clear(&self, u64, String) -> Result<(), Error>;
    }
}

build_rpc_trait! {
    pub trait RpcSelect {
        #[rpc(name = "select_grammar_load")]
        fn load(&self, Vec<String>, Vec<String>) -> Result<u64, Error>;

        #[rpc(name = "select_grammar_unload")]
        fn unload(&self, u64) -> Result<(), Error>;

        #[rpc(name = "select_grammar_activate")]
        fn activate(&self, u64) -> Result<(), Error>;

        #[rpc(name = "select_grammar_deactivate")]
        fn deactivate(&self, u64) -> Result<(), Error>;

        #[rpc(name = "select_grammar_text_set")]
        fn text_set(&self, u64, String) -> Result<(), Error>;

        #[rpc(name = "select_grammar_text_change")]
        fn text_change(&self, u64, u32, u32, String) -> Result<(), Error>;

        #[rpc(name = "select_grammar_text_delete")]
        fn text_delete(&self, u64, u32, u32) -> Result<(), Error>;

        #[rpc(name = "select_grammar_text_insert")]
        fn text_insert(&self, u64, u32, String) -> Result<(), Error>;

        #[rpc(name = "select_grammar_text_get")]
        fn text_get(&self, u64) -> Result<String, Error>;
    }
}

build_rpc_trait! {
    pub trait RpcDictation {
        #[rpc(name = "dictation_grammar_load")]
        fn load(&self) -> Result<u64, Error>;

        #[rpc(name = "dictation_grammar_unload")]
        fn unload(&self, u64) -> Result<(), Error>;

        #[rpc(name = "dictation_grammar_activate")]
        fn activate(&self, u64) -> Result<(), Error>;

        #[rpc(name = "dictation_grammar_deactivate")]
        fn deactivate(&self, u64) -> Result<(), Error>;

        #[rpc(name = "dictation_grammar_context_set")]
        fn context_set(&self, u64, String) -> Result<(), Error>;
    }
}

build_rpc_trait! {
    pub trait RpcCatchall {
        #[rpc(name = "catchall_grammar_load")]
        fn load(&self) -> Result<u64, Error>;

        #[rpc(name = "catchall_grammar_unload")]
        fn unload(&self, u64) -> Result<(), Error>;

        #[rpc(name = "catchall_grammar_activate")]
        fn activate(&self, u64) -> Result<(), Error>;

        #[rpc(name = "catchall_grammar_deactivate")]
        fn deactivate(&self, u64) -> Result<(), Error>;
    }
}

build_rpc_trait! {
    pub trait RpcEngine {
        #[rpc(name = "engine_register")]
        fn register(&self) -> Result<u64, Error>;

        #[rpc(name = "engine_unregister")]
        fn unregister(&self, u64) -> Result<(), Error>;


        #[rpc(name = "microphone_set_state")]
        fn microphone_set_state(&self, MicrophoneState) -> Result<(), Error>;

        #[rpc(name = "microphone_get_state")]
        fn microphone_get_state(&self) -> Result<MicrophoneState, Error>;

        #[rpc(name = "get_current_user")]
        fn get_current_user(&self) -> Result<Option<String>, Error>;
    }
}
