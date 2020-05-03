use diesel::pg::PgConnection;
use std::ops::Deref;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};

use r2d2;
use r2d2_diesel::ConnectionManager;

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn init_pool() -> Pool {
    let manager = ConnectionManager::<PgConnection>::new(database_url());

    r2d2::Pool::builder().build(manager).expect("Failed to create pool")
}

fn database_url() -> String {
    String::from(env!("DATABASE_URL"))
}

pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, Self::Error> {
        let pool = request.guard::<State<Pool>>()?;

        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),

            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}