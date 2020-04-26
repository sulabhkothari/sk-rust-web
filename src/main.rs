#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod other {
    #[get("/world")]
    pub fn world() -> &'static str {
        "Hello, world!"
    }
}

#[get("/hello")]
pub fn hello() -> &'static str {
    "Hello, outside world!"
}

#[get("/hello?wave&<name>")]
fn hello_query_params(name: Option<String>) -> String {
    name.map(|name| format!("Hi, {}!", name))
        .unwrap_or_else(|| "Hello!".into())
}

#[get("/hello/<name>/<age>/<cool>")]
fn hello_combine(name: String, age: u8, cool: bool) -> String {
    format!("{}{}{}", name, age, cool)
}

// By the way, if you were to omit the rank parameter in the user_str or user_int routes, Rocket
// would emit an error and abort launch, indicating that the routes collide, or can match against
// similar incoming requests. The rank parameter resolves this collision.
#[get("/user/<id>")]
fn user(id: usize) -> &'static str {
    "USER"
}

#[get("/user/<id>", rank = 2)]
fn user_int(id: isize) -> &'static str {
    "USER-INT"
}

#[get("/user/<id>", rank = 3)]
fn user_str(id: String) -> &'static str {
    "USER-STR"
}

#[get("/account/<id>", rank = 3)]
fn account(id: Result<i32, &rocket::http::RawStr>) -> &'static str {
    match id {
        Ok(x) => {
            println!("Got int {}", x);
            "Int was valid"
        }
        Err(s) => {
            println!("Got a string");
            "It was a string"
        }
    }
}

//As with paths, you can also match against multiple segments in a query by using <param..>. The
// type of such parameters, known as query guards, must implement the FromQuery trait. Query guards
// must be the final component of a query: any text after a query parameter will result in a
// compile-time error.
use rocket::request::Form;
use rocket::data::FromDataSimple;

#[derive(Debug)]
#[derive(FromForm)]
struct User {
    name: String,
    account: usize,
}

#[get("/item?<id>&<user..>")]
fn item(id: usize, user: Form<User>) -> &'static str {
    println!("{:?}", user);
    "As with paths, you can also match against multiple segments in a query by using <param..>. The type of such parameters, known as query guards, must implement the FromQuery trait. Query guards must be the final component of a query: any text after a query parameter will result in a compile-time error."
}

// The validation policy is implemented through the FromRequest trait. Every type that implements
// FromRequest is a request guard.
// Request guards always fire in left-to-right declaration order
// #[get("/sensitive")]
// fn sensitive(key: ApiKey) { /* .. */ }
// Guard Transparency:
// When a request guard type can only be created through its FromRequest implementation, and the type
// is not Copy, the existence of a request guard value provides a type-level proof that the current
// request has been validated against an arbitrary policy. This provides powerful means of protecting
// your application against access-control violations by requiring data accessing methods to witness
// a proof of authorization via a request guard. We call the notion of using a request guard as a witness
// guard transparency.

use rocket::http::{Cookie, Cookies};

#[get("/")]
fn index(cookies: Cookies) -> Option<String> {
    println!("At Index");
    if let Some(str) = cookies.get("message")
        .map(|value| format!("Message: {}", value)) {
        Some(str)
    } else {
        Some(String::from("Not found a cookie"))
    }
}

use rocket::response::{Flash, Redirect};

/// Retrieve the user's ID, if any.
#[get("/user_id")]
fn user_id(mut cookies: Cookies) -> Option<String> {
    cookies.get_private("user_id")
        .map(|cookie| format!("User ID: {}", cookie.value()))
}

#[get("/set/message")]
fn set_message(mut cookies: Cookies) -> Option<String> {
    let cookie = Cookie::build("message", "Rust cookie is sweet!")
        .path("/")
        .secure(true)
        .finish();
    cookies.add(cookie);
    cookies.add_private(Cookie::build("user_id", "sk").path("/").finish());
    Some(String::from("Added a message"))
}

/// Remove the `user_id` cookie.
#[post("/logout")]
fn logout(mut cookies: Cookies) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named("user_id"));
    Flash::success(Redirect::to("/"), "Successfully logged out.")
}

// To encrypt private cookies, Rocket uses the 256-bit key specified in the secret_key configuration
// parameter. If one is not specified, Rocket will automatically generate a fresh key. Note, however,
// that a private cookie can only be decrypted with the same key with which it was encrypted. As such,
// it is important to set a secret_key configuration parameter when using private cookies so that
// cookies decrypt properly after an application restart.

// openssl rand -base64 32

// For safety reasons, Rocket currently requires that at most one Cookies instance be active at a time.
// std::mem::drop(cookies);

// Only requests where the Content-Type header matches the format parameter will match to the route.
#[post("/user", format = "application/json", data = "<user>")]
fn new_user(user: Form<User>) { /* ... */ }

// When a route indicates a non-payload-supporting method (GET, HEAD, OPTIONS) the format route parameter
// instructs Rocket to check against the Accept header of the incoming request. Only requests where
// the preferred media type in the Accept header matches the format parameter will match to the route.

// When a route indicates a non-payload-supporting method (GET, HEAD, OPTIONS) the format route parameter
// instructs Rocket to check against the Accept header of the incoming request.

// Body data processing, like much of Rocket, is type directed. To indicate that a handler expects
// body data, annotate it with data = "<param>", where param is an argument in the handler. The argument's
// type must implement the FromData trait.
// Any type that implements FromData is also known as a data guard.

#[derive(FromForm)]
struct Task {
    complete: bool,
    description: String,
}

#[post("/todo", data = "<task>")]
fn new(task: Form<Task>) -> Option<String> { Some(String::from("user")) }

// The Form type implements the FromData trait as long as its generic parameter implements the FromForm
// trait. In the example, we've derived the FromForm trait automatically for the Task structure. FromForm
// can be derived for any structure whose fields implement FromFormValue. If a POST /todo request
// arrives, the form data will automatically be parsed into the Task structure. If the data that arrives
// isn't of the correct Content-Type, the request is forwarded.
#[post("/todo2", data = "<task>")]
fn new2(task: Option<Form<Task>>) { /* .. */ }

// A LenientForm<T> will parse successfully from an incoming form as long as the form contains a
// superset of the fields in T. Said another way, a LenientForm<T> automatically discards extra fields
// without error. For instance, if an incoming form contains the fields "a", "b", and "c" while T only
// contains "a" and "c", the form will parse as LenientForm<T>.
use rocket::request::LenientForm;

#[post("/todo3", data = "<task>")]
fn new3(task: LenientForm<Task>) { /* .. */ }

// JSON:
use rocket_contrib::json::Json;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Deserialize)]
struct TaskForJson {
    description: String,
    complete: bool,
}

#[post("/todo4", format = "json", data = "<task>")]
fn new4(task: Json<TaskForJson>) { /* .. */ }
// The only condition is that the generic type in Json implements the Deserialize trait from Serde

#[derive(FromForm)]
struct External {
    // Field Renaming
    #[form(field = "type")]
    api_type: String
}

// Field Validation:
use rocket::http::RawStr;
use rocket::request::FromFormValue;

struct AdultAge(usize);

impl<'v> FromFormValue<'v> for AdultAge {
    type Error = &'v RawStr;

    fn from_form_value(form_value: &'v RawStr) -> Result<AdultAge, &'v RawStr> {
        match form_value.parse::<usize>() {
            Ok(age) if age >= 21 => Ok(AdultAge(age)),
            _ => Err(form_value),
        }
    }
}

#[derive(FromForm)]
struct Person {
    age: AdultAge
}

// The derive generates an implementation of the FromFormValue trait for the decorated enum. The
// implementation returns successfully when the form value matches, case insensitively, the stringified
// version of a variant's name, returning an instance of said variant.
#[derive(FromFormValue)]
enum MyValue {
    First,
    Second,
    Third,
}

// Streaming:
// Sometimes you just want to handle incoming data directly. For example, you might want to stream
// the incoming data out to a file. Rocket makes this as simple as possible via the Data type:
use rocket::Data;

#[post("/upload", format = "plain", data = "<data>")]
fn upload(data: Data) -> Result<String, std::io::Error> {
    data.stream_to_file("/tmp/upload.txt").map(|n| n.to_string())
}

// Error Catchers:
// Routing may fail for a variety of reasons. These include:
//     A guard fails.
//     A handler returns a Responder that fails.
//     No routes matched.
// If any of these conditions occur, Rocket returns an error to the client. To do so, Rocket invokes
// the catcher corresponding to the error's status code. Catchers are similar to routes except in that:
//     Catchers are only invoked on error conditions.
//     Catchers are declared with the catch attribute.
//     Catchers are registered with register() instead of mount().
//     Any modifications to cookies are cleared before a catcher is invoked.
//     Error catchers cannot invoke guards of any sort.
// Rocket provides default catchers for all of the standard HTTP error codes.
// As with routes, the return type (here T) must implement Responder.
use rocket::Request;

#[catch(404)]
fn not_found(req: &Request) -> String {
    format!("Sorry, '{}' is not a valid path.", req.uri())
}

#[get("/person/<name>?<age>")]
fn person(name: String, age: Option<u8>) { /* .. */ }

//use rocket_contrib::templates::Template;
fn main() {
    // rustup override set nightly
    rocket::ignite().mount("/", routes![hello, other::world, user, user_int, user_str, account, item, index, user_id, logout, set_message])
        //.attach(Template::fairing())
        .launch();
    rocket::ignite().register(catchers![not_found]);

    // with unnamed parameters, in route path declaration order
    let mike = uri!(person: "Mike Smith", 28);
    assert_eq!(mike.to_string(), "/person/Mike%20Smith?age=28");

    // with named parameters, order irrelevant
    let mike = uri!(person: name = "Mike", age = 28);
    let mike = uri!(person: age = 28, name = "Mike");
    assert_eq!(mike.to_string(), "/person/Mike?age=28");

    // with a specific mount-point
    let mike = uri!("/api", person: name = "Mike", age = 28);
    assert_eq!(mike.to_string(), "/api/person/Mike?age=28");

    // with optional (defaultable) query parameters ignored
    let mike = uri!(person: "Mike", _);
    let mike = uri!(person: name = "Mike", age = _);
    assert_eq!(mike.to_string(), "/person/Mike");

    verify_add_user_uri();
}

// If your Rocket application suddenly stops building, ensure you're using the latest version of Rust
// nightly and Rocket by updating your toolchain and dependencies with:
// rustup update && cargo update

// RESPONSE:
use rocket::response::NamedFile;
use rocket::response::status::NotFound;
use std::path::PathBuf;
use std::path::Path;

#[get("/<file..>")]
fn files(file: PathBuf) -> Result<NamedFile, NotFound<String>> {
    let path = Path::new("static/").join(file);
    NamedFile::open(&path).map_err(|e| NotFound(e.to_string()))
}

#[get("/<file..>")]
fn files2(file: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new("static/").join(file)).ok()
}

use std::os::unix::net::UnixStream;
use rocket::response::Stream;

#[get("/stream")]
fn stream() -> Result<Stream<UnixStream>, std::io::Error> {
    UnixStream::connect("/path/to/my/socket").map(Stream::from)
}

use serde::Serialize;

#[derive(Serialize)]
struct TaskEmblem {
    id: i32
}

// The Json type serializes the structure into JSON, sets the Content-Type to JSON, and emits the
// serialized data in a fixed-sized body. If serialization fails, a 500 - Internal Server Error is returned.
#[get("/todo")]
fn todo() -> Json<TaskEmblem> {
    Json(TaskEmblem { id: 90 })
}

// #[get("/")]
// fn index_more() -> Template {
//     let context = vector![1,2,3];
//     Template::render("index", &context)
// }

// The UriDisplay trait can be derived for custom types. For types that appear in the path part of a
// URI, derive using UriDisplayPath; for types that appear in the query part of a URI, derive using
// UriDisplayQuery.
#[derive(FromForm, UriDisplayQuery)]
struct UserDetails<'r> {
    age: Option<usize>,
    nickname: &'r RawStr,
}

#[post("/user/<id>?<details..>")]
fn add_user(id: usize, details: Form<UserDetails>) { /* .. */ }

#[get("/person/<id>?<details..>")]
fn person_optional(id: usize, details: Option<Form<UserDetails>>) { /* .. */ }

// By deriving using UriDisplayQuery, an implementation of UriDisplay<Query> is automatically generated,
// allowing for URIs to add_user to be generated using uri!:
fn verify_add_user_uri() {
    let link = uri!(add_user: 120, UserDetails { age: Some(20), nickname: "Bob".into() });
    assert_eq!(link.to_string(), "/user/120?age=20&nickname=Bob");

    // impl<'a, P: UriPart> FromUriParam<P, &'a str> for String {
    //     type Target = &'a str;
    // }
    // Conversions nest. For instance, a value of type T can be supplied when a value of type Option<Form<T>>
    // is expected:
    uri!(person_optional: id = 100, details = UserDetails { age: Some(20), nickname: "Bob".into() });
}

//https://docs.rs/rocket_oauth2/0.2.0/rocket_oauth2/
//https://docs.rs/rayon/1.3.0/rayon/
//http://blog.jecrooks.com/posts/rust-on-appengine.html
//https://docs.rs/tokio/0.2.18/tokio/