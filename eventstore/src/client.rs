use std::{
    sync::Arc,
    time::Instant,
};

use eventstore_client::{
    AppendToStreamOptions,
    Client,
    ExpectedRevision,
    ReadStreamOptions,
    ResolvedEvent,
    StreamPosition,
    SubscribeToStreamOptions,
    ToEvents,
};
use tokio::sync::mpsc;
use tracing::Instrument;

use crate::handler::{
    Handler,
    HandlerError,
};

pub struct EventStoreClient {
    client: Arc<Client>,
}

impl EventStoreClient {
    pub fn new(connection_string: String) -> Self {
        let settings = connection_string
            .parse()
            .expect("connection string should be correct");

        let client = Client::new(settings).expect("client should be created");

        Self {
            client: Arc::new(client),
        }
    }

    pub async fn read_stream(
        &self,
        stream_name: String,
        stream_position: StreamPosition<u64>,
    ) -> Vec<ResolvedEvent> {
        let mut stream = self
            .client
            .read_stream(
                stream_name,
                &ReadStreamOptions::default().position(stream_position),
            )
            .await
            .expect("could not read stream");

        let mut events = Vec::new();

        while let Ok(Some(event)) = stream.next().await {
            events.push(event)
        }

        events
    }

    pub async fn subscribe_to_stream(
        &self,
        stream_name: String,
        handler: impl Handler + Send + Sync + 'static,
    ) {
        tracing::debug!(
            "subscribing to stream: {} -> {}",
            stream_name,
            handler.name()
        );

        let client = self.client.clone();

        let mut subscription = client
            .subscribe_to_stream(
                stream_name.clone(),
                &SubscribeToStreamOptions::default()
                    .resolve_link_tos()
                    .start_from(StreamPosition::Start),
            )
            .await;

        let (sender, mut receiver) = mpsc::channel::<ResolvedEvent>(1);

        let _handle = tokio::spawn(async move {
            'listener: loop {
                let event = receiver.recv().await.unwrap();
                let recorded_event = event.event.as_ref().unwrap();
                let span = tracing::info_span!(
                    "es_event_handler",
                    stream_name,
                    event_id = %recorded_event.id,
                    event_type = recorded_event.event_type);

                let mut current_delivery_attempt = 0;

                'handler: loop {
                    current_delivery_attempt += 1;

                    span.clone().in_scope(|| {
                        tracing::debug!("handling event, attempt {}", current_delivery_attempt);
                    });

                    let start_time = Instant::now();

                    let handler_result = async { handler.handle(&event).await }
                        .instrument(span.clone())
                        .await;

                    if let Err(error) = handler_result {
                        match error {
                            HandlerError::Retryable => {
                                if current_delivery_attempt < 5 {
                                    tracing::warn!("failed to handle event with a retryable error");
                                    continue 'handler;
                                } else {
                                    tracing::error!("failed to handle event with a retryable error but the maximum number of retries has been reached, unsubscribing from stream");
                                    break 'listener;
                                }
                            }
                            HandlerError::Unrecoverable => {
                                tracing::error!("failed to handle event with an unrecoverable error, unsubscribing from stream");
                                break 'listener;
                            }
                        }
                    }

                    span.clone().in_scope(|| {
                        tracing::debug!("event handled in {:?}", start_time.elapsed());
                    });

                    break 'handler;
                }
            }
        });

        loop {
            let event_result = subscription.next().await;

            if event_result.is_err() {
                break;
            }

            let send_result = sender.send(event_result.unwrap()).await;

            if send_result.is_err() {
                break;
            }
        }
    }

    pub async fn append_to_stream(
        &self,
        stream_name: String,
        expected_revision: ExpectedRevision,
        events: impl ToEvents,
    ) {
        let _result = self
            .client
            .append_to_stream(
                stream_name,
                &AppendToStreamOptions::default().expected_revision(expected_revision),
                events,
            )
            .await;
    }
}
