use rustlink::grammar::Grammar;
use errors::Error;
use rustlink::engine::MicrophoneState;

build_rpc_trait! {
    pub trait Rpc {
        #[rpc(name = "grammar_load")]
        fn grammar_load(&self, Grammar, bool) -> Result<u64, Error>;

        #[rpc(name = "grammar_unload")]
        fn grammar_unload(&self, u64) -> Result<(), Error>;

        #[rpc(name = "grammar_rule_activate")]
        fn grammar_rule_activate(&self, u64, String) -> Result<(), Error>;

        #[rpc(name = "grammar_rule_deactivate")]
        fn grammar_rule_deactivate(&self, u64, String) -> Result<(), Error>;

        #[rpc(name = "grammar_list_append")]
        fn grammar_list_append(&self, u64, String, String) -> Result<(), Error>;

        #[rpc(name = "grammar_list_remove")]
        fn grammar_list_remove(&self, u64, String, String) -> Result<(), Error>;

        #[rpc(name = "grammar_list_clear")]
        fn grammar_list_clear(&self, u64, String) -> Result<(), Error>;

        #[rpc(name = "engine_register")]
        fn engine_register(&self) -> Result<u64, Error>;

        #[rpc(name = "engine_unregister")]
        fn engine_unregister(&self, u64) -> Result<(), Error>;

        #[rpc(name = "microphone_set_state")]
        fn microphone_set_state(&self, MicrophoneState) -> Result<(), Error>;

        #[rpc(name = "microphone_get_state")]
        fn microphone_get_state(&self) -> Result<MicrophoneState, Error>;
    }
}
