use diesel::prelude::*;
use diesel::{Connection, SqliteConnection};
use dotenvy::dotenv;
use example::todo_service_server::{TodoService, TodoServiceServer};
use example::{Empty, TodoId, TodoList};
use models::NewTodo;
use std::env;
use std::time::Duration;
use tokio::net::TcpListener;
use tonic::{transport::Server, Request, Response, Status};
use tower_http::cors::{Any, CorsLayer};
use logcall::logcall;

pub mod example {
    tonic::include_proto!("example");
}

pub mod models;
pub mod schema;

pub fn establish_connection() -> SqliteConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("Connecting to {}", database_url);
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

impl From<example::Todo> for models::Todo {
    fn from(todo: example::Todo) -> Self {
        models::Todo {
            id: todo.id as i32,
            title: todo.title,
            description: Some(todo.description),
            completed: todo.completed,
        }
    }
}

impl From<&example::Todo> for models::Todo {
    fn from(todo: &example::Todo) -> Self {
        models::Todo {
            id: todo.id as i32,
            title: todo.title.clone(),
            description: Some(todo.description.clone()),
            completed: todo.completed,
        }
    }
}

impl From<example::Todo> for models::NewTodo {
    fn from(todo: example::Todo) -> Self {
        models::NewTodo {
            title: todo.title,
            description: Some(todo.description),
            completed: todo.completed,
        }
    }
}

impl From<models::Todo> for example::Todo {
    fn from(todo: models::Todo) -> Self {
        example::Todo {
            id: todo.id as i64,
            title: todo.title,
            description: todo.description.unwrap_or_else(|| "".to_string()),
            completed: todo.completed,
        }
    }
}

impl From<&models::Todo> for example::Todo {
    fn from(todo: &models::Todo) -> Self {
        example::Todo {
            id: todo.id as i64,
            title: todo.title.clone(),
            description: todo.description.clone().unwrap_or_else(|| "".to_string()),
            completed: todo.completed,
        }
    }
}

// TODO unwraps -> good error handling
// impl From<diesel::result::Error> for Status {
//     fn from(err: diesel::result::Error) -> Self {
//         Status {}
//     }
// }

#[derive(Default)]
pub struct TodoBase {}

#[tonic::async_trait]
impl TodoService for TodoBase {
    
    #[logcall("info")]
    async fn update_todo(
        &self,
        request: Request<example::Todo>,
    ) -> Result<Response<example::Todo>, Status> {
        let mut connection = establish_connection();

        let record: models::Todo = request.into_inner().into();

        let result = diesel::update(crate::schema::todos::dsl::todos.find(record.id))
            .set((
                crate::schema::todos::dsl::title.eq(record.title),
                crate::schema::todos::dsl::description.eq(record.description),
                crate::schema::todos::dsl::completed.eq(record.completed),
            ))
            .returning(models::Todo::as_returning())
            .get_result(&mut connection)
            .unwrap();

        Ok(Response::new(result.into()))
    }
    
    #[logcall("info")]
    async fn delete_todo(&self, request: Request<TodoId>) -> Result<Response<Empty>, Status> {
        let mut connection = establish_connection();

        let req_id = request.into_inner().id as i32;

        let num_deleted = diesel::delete(
            crate::schema::todos::dsl::todos.filter(crate::schema::todos::dsl::id.eq(req_id)),
        )
        .execute(&mut connection)
        .unwrap();

        debug_assert_eq!(1, num_deleted);

        Ok(Response::new(Empty {}))
    }

    #[logcall("info")]
    async fn add_todo(
        &self,
        request: Request<example::Todo>,
    ) -> Result<Response<example::Todo>, Status> {
        let mut connection = establish_connection();
        let record: NewTodo = request.into_inner().into();

        let result: models::Todo = diesel::insert_into(schema::todos::table)
            .values(&record)
            .returning(models::Todo::as_returning())
            .get_result(&mut connection)
            .unwrap();

        Ok(Response::new(result.into()))
    }

    #[logcall("info")]
    async fn get_todos(&self, _request: Request<Empty>) -> Result<Response<TodoList>, Status> {
        let mut connection = establish_connection();

        let results: Vec<models::Todo> = crate::schema::todos::dsl::todos
            .select(models::Todo::as_select())
            .load(&mut connection)
            .unwrap();

        let response = TodoList {
            todos: results.iter().map(|x| x.into()).collect(),
        };

        Ok(Response::new(response))
    }
}

pub async fn serve(listener: TcpListener) -> Result<(), tonic::transport::Error> {
    Server::builder()
        .accept_http1(true)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(Any)
                .allow_methods(Any)
                .max_age(Duration::from_secs(60) * 30),
        )
        .layer(tonic_web::GrpcWebLayer::new())
        .add_service(TodoServiceServer::new(TodoBase::default()))
        .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
        .await
}
