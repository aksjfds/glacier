use std::future::Future;

#[allow(async_fn_in_trait)]
pub trait AsyncMapExt<F, Fut, E, O> {
    fn async_map(self, f: F) -> impl Future<Output = Result<O, E>>;
}

impl<T, F, Fut, E, O> AsyncMapExt<F, Fut, E, O> for Result<T, E>
where
    F: Fn(T) -> Fut,
    Fut: Future<Output = O>,
{
    async fn async_map(self, f: F) -> Result<O, E> {
        match self {
            Ok(req) => Ok(f(req).await),
            Err(e) => Err(e),
        }
    }
}
