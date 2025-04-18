use crate::{request::Request, response::Response};

#[allow(unused)]
#[allow(async_fn_in_trait)]
pub trait HandleReq<E>
where
    Self: Sized,
{
    fn handle(self, req: Request) -> Result<Response, E> {
        todo!()
    }
    fn async_handle(self, req: Request) -> impl Future<Output = Result<Response, E>> {
        async { todo!() }
    }

    fn filter(self, req: Request) -> Result<Request, E> {
        todo!()
    }
    fn async_filter(self, req: Request) -> impl Future<Output = Result<Request, E>> {
        async { todo!() }
    }
}
