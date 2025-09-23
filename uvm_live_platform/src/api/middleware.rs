/// Generic middleware trait that can process any type of request
pub trait Middleware<Opts, Res, Err> {
    /// Process a request, optionally calling the next middleware in the chain
    fn process(
        &self,
        options: &Opts,
        next: &dyn Fn(&Opts) -> Result<Res, Err>,
    ) -> Result<Res, Err>;
}

// Type aliases for convenience and backward compatibility
#[cfg(feature = "cache")]
pub type FetchReleaseMiddleware = dyn Middleware<
    crate::api::fetch_release::FetchReleaseOptions,
    crate::Release,
    crate::error::FetchReleaseError
>;

/// A generic middleware chain that processes requests through multiple middlewares
pub struct MiddlewareChain<'a, Opts, Res, Err> {
    middlewares: Vec<Box<dyn Middleware<Opts, Res, Err> + Send + Sync + 'a>>,
}

impl<'a, Opts, Res, Err> std::fmt::Debug for MiddlewareChain<'a, Opts, Res, Err> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiddlewareChain")
            .field("middleware_count", &self.middlewares.len())
            .finish()
    }
}

impl<'a, Opts, Res, Err> MiddlewareChain<'a, Opts, Res, Err> {
    /// Create a new empty middleware chain
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    /// Add a middleware to the chain
    pub fn add<M: Middleware<Opts, Res, Err> + Send + Sync + 'a>(mut self, middleware: M) -> Self {
        self.middlewares.push(Box::new(middleware));
        self
    }

    /// Execute the middleware chain with the given options and final handler
    pub fn execute<F>(
        &self,
        options: &Opts,
        final_handler: F,
    ) -> Result<Res, Err>
    where
        F: Fn(&Opts) -> Result<Res, Err> + Send + Sync,
    {
        self.execute_recursive(options, &final_handler, 0)
    }

    fn execute_recursive<F>(
        &self,
        options: &Opts,
        final_handler: &F,
        index: usize,
    ) -> Result<Res, Err>
    where
        F: Fn(&Opts) -> Result<Res, Err> + Send + Sync,
    {
        if index >= self.middlewares.len() {
            // No more middlewares, call the final handler
            final_handler(options)
        } else {
            // Call the current middleware with a next function that continues the chain
            let next = |opts: &Opts| -> Result<Res, Err> {
                self.execute_recursive(opts, final_handler, index + 1)
            };
            self.middlewares[index].process(options, &next)
        }
    }
}

impl<'a, Opts, Res, Err> Default for MiddlewareChain<'a, Opts, Res, Err> {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::fetch_release::FetchReleaseOptions;
    use crate::{Release, error::FetchReleaseError};

    struct TestMiddleware {
        name: String,
        calls: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl TestMiddleware {
        fn new(name: &str, calls: std::sync::Arc<std::sync::Mutex<Vec<String>>>) -> Self {
            Self {
                name: name.to_string(),
                calls,
            }
        }
    }

    impl Middleware<FetchReleaseOptions, Release, FetchReleaseError> for TestMiddleware {
        fn process(
            &self,
            options: &FetchReleaseOptions,
            next: &dyn Fn(&FetchReleaseOptions) -> Result<Release, FetchReleaseError>,
        ) -> Result<Release, FetchReleaseError> {
            self.calls.lock().unwrap().push(format!("{}_before", self.name));
            let result = next(options);
            self.calls.lock().unwrap().push(format!("{}_after", self.name));
            result
        }
    }

    #[test]
    fn test_middleware_chain_execution_order() {
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        
        let chain: MiddlewareChain<FetchReleaseOptions, Release, FetchReleaseError> = MiddlewareChain::new()
            .add(TestMiddleware::new("middleware1", calls.clone()))
            .add(TestMiddleware::new("middleware2", calls.clone()));

        let options = FetchReleaseOptions {
            version: "test".to_string(),
            architecture: vec![],
            platform: vec![],
            stream: vec![],
            entitlements: vec![],
        };

        // Mock final handler that records the call
        let final_handler = |_: &FetchReleaseOptions| -> Result<Release, FetchReleaseError> {
            calls.lock().unwrap().push("final_handler".to_string());
            Err(FetchReleaseError::CacheError("test error".to_string()))
        };

        let _ = chain.execute(&options, final_handler);

        let recorded_calls = calls.lock().unwrap();
        assert_eq!(
            *recorded_calls,
            vec![
                "middleware1_before",
                "middleware2_before", 
                "final_handler",
                "middleware2_after",
                "middleware1_after"
            ]
        );
    }

    // Test with a different type to demonstrate generics
    #[derive(Debug)]
    struct CustomOptions {
        pub name: String,
    }

    #[derive(Debug)]
    struct CustomResult {
        pub output: String,
    }

    #[derive(Debug, thiserror::Error)]
    #[error("Custom error: {message}")]
    struct CustomError {
        message: String,
    }

    struct CustomMiddleware;

    impl Middleware<CustomOptions, CustomResult, CustomError> for CustomMiddleware {
        fn process(
            &self,
            options: &CustomOptions,
            next: &dyn Fn(&CustomOptions) -> Result<CustomResult, CustomError>,
        ) -> Result<CustomResult, CustomError> {
            println!("Processing custom options: {}", options.name);
            next(options)
        }
    }

    #[test]
    fn test_generic_middleware() {
        let chain: MiddlewareChain<CustomOptions, CustomResult, CustomError> = MiddlewareChain::new()
            .add(CustomMiddleware);

        let options = CustomOptions {
            name: "test".to_string(),
        };

        let final_handler = |opts: &CustomOptions| -> Result<CustomResult, CustomError> {
            Ok(CustomResult {
                output: format!("Processed: {}", opts.name),
            })
        };

        let result = chain.execute(&options, final_handler).unwrap();
        assert_eq!(result.output, "Processed: test");
    }
}
