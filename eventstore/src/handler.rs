use async_trait::async_trait;
use eventstore::ResolvedEvent;

pub type HandlerResult = Result<(), HandlerError>;

#[derive(Debug)]
pub enum HandlerError {
    Retryable,
    Unrecoverable,
}

#[async_trait]
pub trait Handler {
    fn name(&self) -> String;

    async fn handle(&self, event: &ResolvedEvent) -> HandlerResult;
}
