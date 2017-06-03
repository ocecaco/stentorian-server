use rustlink::grammar::Grammar;
use errors::Error;

build_rpc_trait! {
    pub trait Rpc {
        #[rpc(name = "grammar_load")]
        fn grammar_load(&self, Grammar, bool) -> Result<u64, Error>;

        #[rpc(name = "grammar_unload")]
        fn grammar_unload(&self, u64) -> Result<(), Error>;
    }
}
