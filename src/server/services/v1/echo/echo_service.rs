use std::sync::{Arc};
use autometrics::autometrics;
use tonic::{Request, Response, Status};

use crate::database::{CacheClient, get_connection, PgPool};

use autometrics::objectives::{
    Objective, ObjectiveLatency, ObjectivePercentile
};
use protos::echo::v1::echo_service_server::EchoService;
use protos::echo::v1::{UnaryEchoRequest, UnaryEchoResponse};
use crate::server::services::v1::echo::echo_handlers::echo;

const API_SLO: Objective = Objective::new("api")
    .success_rate(ObjectivePercentile::P99_9)
    .latency(ObjectiveLatency::Ms250, ObjectivePercentile::P99);

pub struct EchoServiceServerImpl {
    pub pool: Arc<PgPool>,
    pub cache: CacheClient,
}

impl Clone for EchoServiceServerImpl {
    fn clone(&self) -> Self {
        EchoServiceServerImpl {
            pool: Arc::clone(&self.pool),
            cache: self.cache.clone(),
        }
    }
}

impl EchoServiceServerImpl {
    pub(crate) fn new(pool: Arc<PgPool>, cache: CacheClient) -> Self {
        EchoServiceServerImpl {
            pool,
            cache,
        }
    }
}

#[tonic::async_trait]
#[autometrics(objective = API_SLO)]
impl EchoService for EchoServiceServerImpl {
    async fn unary_echo(&self, request: Request<UnaryEchoRequest>) -> Result<Response<UnaryEchoResponse>, Status> {
        let mut conn = get_connection(&self.pool).await?;
        let inner_request = request.into_inner();

        self.cache.handle_cache("unary_echo", &inner_request.clone(), || {
            async move {
                echo(inner_request, &mut conn)
                    .await
                    .map(Response::new)
                    .map_err(|e| e.into())
            }
        }).await
    }
}