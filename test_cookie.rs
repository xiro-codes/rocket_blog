use rocket::http::Cookie;

fn main() {
    let token = "test";
    let c = Cookie::build(("token", token.to_string())).path("/").build();
    println!("{:?}", c);
}
