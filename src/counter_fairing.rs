use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};

use rocket::{Request, Data, Response};
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{Method, ContentType, Status};

#[derive(Default)]
pub struct Counter {
    get: AtomicUsize,
    post: AtomicUsize,
}


impl Counter {
    pub fn new(get: usize, post: usize) -> Counter {
        Counter {
            get: AtomicUsize::new(get),
            post: AtomicUsize::new(post),
        }
    }
}

impl Fairing for Counter {
    fn info(&self) -> Info {
        Info {
            name: "GET/POST Counter",
            kind: Kind::Request | Kind::Response,
        }
    }

    fn on_request(&self, request: &mut Request, _: &Data) {
        if request.method() == Method::Get {
            self.get.fetch_add(1, Ordering::Relaxed);
        } else if request.method() == Method::Post {
            self.post.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn on_response(&self, request: &Request, response: &mut Response) {
        // Don't change a successful user's response, ever.
        if response.status() != Status::NotFound {
            return;
        }

        if request.method() == Method::Get && request.uri().path() == "/counts" {
            let get_count = self.get.load(Ordering::Relaxed);
            let post_count = self.post.load(Ordering::Relaxed);

            let body = format!("Get: {}\nPost: {}", get_count, post_count);
            response.set_status(Status::Ok);
            response.set_header(ContentType::Plain);
            response.set_sized_body(Cursor::new(body));
        }
    }
}