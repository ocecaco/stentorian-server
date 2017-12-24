use errors::Error;
use stentorian::engine::MicrophoneState;
use stentorian::grammar::Grammar;

build_rpc_trait! {
    pub trait Rpc {
        #[rpc(name = "command_grammar_load")]
        fn command_grammar_load(&self, Grammar) -> Result<u64, Error>;

        #[rpc(name = "command_grammar_unload")]
        fn command_grammar_unload(&self, u64) -> Result<(), Error>;

        #[rpc(name = "command_grammar_rule_activate")]
        fn command_grammar_rule_activate(&self, u64, String) -> Result<(), Error>;

        #[rpc(name = "command_grammar_rule_deactivate")]
        fn command_grammar_rule_deactivate(&self, u64, String) -> Result<(), Error>;

        #[rpc(name = "command_grammar_list_append")]
        fn command_grammar_list_append(&self, u64, String, String) -> Result<(), Error>;

        #[rpc(name = "command_grammar_list_remove")]
        fn command_grammar_list_remove(&self, u64, String, String) -> Result<(), Error>;

        #[rpc(name = "command_grammar_list_clear")]
        fn command_grammar_list_clear(&self, u64, String) -> Result<(), Error>;


        #[rpc(name = "select_grammar_load")]
        fn select_grammar_load(&self, Vec<String>, Vec<String>) -> Result<u64, Error>;

        #[rpc(name = "select_grammar_unload")]
        fn select_grammar_unload(&self, u64) -> Result<(), Error>;

        #[rpc(name = "select_grammar_activate")]
        fn select_grammar_activate(&self, u64) -> Result<(), Error>;

        #[rpc(name = "select_grammar_deactivate")]
        fn select_grammar_deactivate(&self, u64) -> Result<(), Error>;

        #[rpc(name = "select_grammar_text_set")]
        fn select_grammar_text_set(&self, u64, String) -> Result<(), Error>;

        #[rpc(name = "select_grammar_text_change")]
        fn select_grammar_text_change(&self, u64, u32, u32, String) -> Result<(), Error>;

        #[rpc(name = "select_grammar_text_delete")]
        fn select_grammar_text_delete(&self, u64, u32, u32) -> Result<(), Error>;

        #[rpc(name = "select_grammar_text_insert")]
        fn select_grammar_text_insert(&self, u64, u32, String) -> Result<(), Error>;

        #[rpc(name = "select_grammar_text_get")]
        fn select_grammar_text_get(&self, u64) -> Result<String, Error>;


        #[rpc(name = "dictation_grammar_load")]
        fn dictation_grammar_load(&self) -> Result<u64, Error>;

        #[rpc(name = "dictation_grammar_unload")]
        fn dictation_grammar_unload(&self, u64) -> Result<(), Error>;

        #[rpc(name = "dictation_grammar_activate")]
        fn dictation_grammar_activate(&self, u64) -> Result<(), Error>;

        #[rpc(name = "dictation_grammar_deactivate")]
        fn dictation_grammar_deactivate(&self, u64) -> Result<(), Error>;

        #[rpc(name = "dictation_grammar_context_set")]
        fn dictation_grammar_context_set(&self, u64, String) -> Result<(), Error>;


        #[rpc(name = "catchall_grammar_load")]
        fn catchall_grammar_load(&self) -> Result<u64, Error>;

        #[rpc(name = "catchall_grammar_unload")]
        fn catchall_grammar_unload(&self, u64) -> Result<(), Error>;

        #[rpc(name = "catchall_grammar_activate")]
        fn catchall_grammar_activate(&self, u64) -> Result<(), Error>;

        #[rpc(name = "catchall_grammar_deactivate")]
        fn catchall_grammar_deactivate(&self, u64) -> Result<(), Error>;


        #[rpc(name = "engine_register")]
        fn engine_register(&self) -> Result<u64, Error>;

        #[rpc(name = "engine_unregister")]
        fn engine_unregister(&self, u64) -> Result<(), Error>;


        #[rpc(name = "microphone_set_state")]
        fn microphone_set_state(&self, MicrophoneState) -> Result<(), Error>;

        #[rpc(name = "microphone_get_state")]
        fn microphone_get_state(&self) -> Result<MicrophoneState, Error>;

        #[rpc(name = "get_current_user")]
        fn get_current_user(&self) -> Result<Option<String>, Error>;
    }
}
