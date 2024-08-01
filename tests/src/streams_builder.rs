use streams_core::nats::{ConnId, NatsConn};

static URL: &str = "nats://localhost:4222";
enum UserRole {
    Admin,
    Public,
}

pub struct TestSetupBuilder {
    pub conn: NatsConn,
}

impl TestSetupBuilder {
    async fn setup(
        conn_id: Option<ConnId>,
        role: UserRole,
    ) -> anyhow::Result<Self> {
        let conn_id = conn_id.unwrap_or(ConnId::rnd());
        let conn = match role {
            UserRole::Public => NatsConn::as_public(URL, conn_id).await?,
            UserRole::Admin => NatsConn::as_admin(URL, conn_id).await?,
        };

        Ok(Self { conn })
    }

    pub async fn as_admin(conn_id: Option<ConnId>) -> anyhow::Result<Self> {
        Self::setup(conn_id, UserRole::Admin).await
    }

    pub async fn as_public(conn_id: Option<ConnId>) -> anyhow::Result<Self> {
        Self::setup(conn_id, UserRole::Public).await
    }

    pub fn conn_id(&self) -> ConnId {
        self.conn.client().conn_id
    }
}
