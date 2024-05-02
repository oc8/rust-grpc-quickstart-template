use std::sync::{Arc};
use autometrics::autometrics;
use tonic::{Code, Request, Response, Status};

use protos::grpc::examples::echo::{echo_server::Echo, EchoRequest, EchoResponse};
use crate::database::{PgPool, PgPooledConnection};
use crate::{errors, rpcs};

use autometrics::objectives::{
    Objective, ObjectiveLatency, ObjectivePercentile
};

const API_SLO: Objective = Objective::new("api")
    .success_rate(ObjectivePercentile::P99_9)
    .latency(ObjectiveLatency::Ms250, ObjectivePercentile::P99);

pub struct EchoService {
    pub pool: Arc<PgPool>,
    pub r_client: redis::Client,
}

impl Clone for EchoService {
    fn clone(&self) -> Self {
        EchoService {
            pool: self.pool.clone(),
            r_client: self.r_client.clone(),
        }
    }
}

#[tonic::async_trait]
#[autometrics(objective = API_SLO)]
impl Echo for EchoService {
    async fn unary_echo(
        &self,
        request: Request<EchoRequest>,
    ) -> Result<Response<EchoResponse>, Status> {
        let mut conn = get_connection(&self.pool)?;
        let mut r_conn = get_redis_connection(&self.r_client)?;
        rpcs::echo(request.into_inner(), &mut conn, &mut r_conn).map(Response::new)
    }
}

pub fn get_connection(pool: &PgPool) -> Result<PgPooledConnection, Status> {
    match pool.get() {
        Err(_) => Err(Status::new(Code::DataLoss, errors::DATABASE_CONNECTION_FAILURE)),
        Ok(conn) => Ok(conn),
    }
}

fn get_redis_connection(r_client: &redis::Client) -> Result<redis::Connection, Status> {
    match r_client.get_connection() {
        Err(_) => Err(Status::new(Code::DataLoss, errors::REDIS_CONNECTION_FAILURE)),
        Ok(conn) => Ok(conn),
    }
}
