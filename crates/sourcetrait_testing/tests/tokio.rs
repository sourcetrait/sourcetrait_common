#![allow(unexpected_cfgs)]

#[cfg(all(test, feature = "test_all"))]
mod tests {
    use sourcetrait_testing::prelude::*;
    
    #[tested(tokio)]
    async fn test_tokio() {
        let _foo = function_name!();
    }
    
    #[tested(tokio(unstable))]
    async fn test_tokio_unstable() {
        let _foo = function_name!();
    }
    
    #[tested(tokio(unstable, flavor = "local"))]
    async fn test_tokio_unstable_double() {
        let _foo = function_name!();
    }
    
    #[tested(tokio(flavor = "multi_thread"))]
    async fn test_tokio_single() {
        let _foo = function_name!();
    }
    
    #[tested(tokio(flavor = "multi_thread", worker_threads = 2))]
    async fn test_tokio_double() {
        let _foo = function_name!();
    }
    
    #[tested]
    fn test_normal() {
        let _foo = function_name!();
    }
}