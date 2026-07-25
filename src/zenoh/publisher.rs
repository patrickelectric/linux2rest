use tracing::*;
use zenoh::Session;

pub struct Publisher {
    publisher: zenoh::pubsub::Publisher<'static>,
}

impl Publisher {
    pub async fn declare(
        session: &Session,
        key_expr: impl Into<String>,
        encoding: zenoh::bytes::Encoding,
    ) -> Self {
        let publisher = session
            .declare_publisher(key_expr.into())
            .encoding(encoding)
            .await
            .unwrap();

        Self { publisher }
    }

    pub fn key_expr(&self) -> &str {
        self.publisher.key_expr().as_str()
    }

    pub async fn put<T>(&self, payload: T)
    where
        T: Into<zenoh::bytes::ZBytes>,
    {
        if let Err(error) = self.publisher.put(payload).await {
            error!(
                key_expr = %self.key_expr(),
                "Failed to put zenoh sample: {error:?}"
            );
        }
    }
}
