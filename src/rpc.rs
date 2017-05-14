use rustlink::grammar::Grammar;
use jsonrpc_core::Error;

build_rpc_trait! {
    pub trait Rpc {
        #[rpc(name = "grammar_load")]
        fn rpc_grammar_load(&self, Grammar, bool) -> Result<String, Error>;

        #[rpc(name = "grammar_unload")]
        fn rpc_grammar_unload(&self, String) -> Result<(), Error>;

    }
}
