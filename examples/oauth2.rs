#![allow(unused_braces)]

use std::time::Duration;

use ft_rs::{models::token::AccessToken, FtClient};
use once_cell::sync::Lazy;
use render::html;
use rocket::{get, http::{Cookie, CookieJar}, response::{content::RawHtml, Redirect}, routes, time::OffsetDateTime, uri};

const CALLBACK_URL: &str = "http://localhost:1337/oauth/callback";
const SCOPES: &[&str] = &["public", "projects", "profile"];
static CLIENT: Lazy<FtClient> = Lazy::new(|| {
  FtClient::from_app(
    std::env::var("FT_RS_TEST_UID").expect("FT_RS_TEST_UID not set"), 
    std::env::var("FT_RS_TEST_SECRET").expect("FT_RS_TEST_SECRET not set")
  ).expect("Couldn't build the client")
});


#[derive(rocket::response::Responder)]
#[response(status = 303)]
struct CustomRedirect<'a> {
  inner: Redirect,
  access_token: Cookie<'a>,
}

#[get("/")]
async fn index(cookies: &CookieJar<'_>) -> RawHtml<String> {
  if let Some(token) = cookies.get("access_token") {
    let token = token.value();
    let token = serde_json::from_str::<AccessToken>(token).unwrap();
    let user = CLIENT.fetch_user_data(&token).await.unwrap();

    RawHtml(html! {
      <div>
        <h1>{"Hello, world!"}</h1>
        <p>{"You are logged in as "}{user.login}{"."}</p>
        <img src={&user.image.link} width={"500"} alt={"Profile Picture"} />
      </div>
    })
  } else {
    let auth_url = CLIENT.get_authorization_url(CALLBACK_URL, SCOPES).unwrap();
    RawHtml(html! {
      <div>
        <h1>{"Hello, world!"}</h1>
        <a href={&auth_url}>{"Login"}</a>
      </div>
    })
  }
}

#[get("/oauth/callback?<code>")]
async fn callback<'a>(code: String) -> CustomRedirect<'a> {
  let token = CLIENT.fetch_access_token(&code, CALLBACK_URL).await.unwrap();
  let token = serde_json::to_string(&token).unwrap();

  CustomRedirect { 
    inner: Redirect::to(uri!(index)), 
    access_token: Cookie::build(("access_token", token))
      .path("/")
      .expires(OffsetDateTime::now_utc() + Duration::from_secs(15 * 60))
      .http_only(true)
      .build()
  }
}

#[tokio::main]
async fn main() {
  let config = rocket::Config {
    port: 1337,
    ..Default::default()
  };

  let _ = rocket::custom(config)
    .mount("/", routes![index, callback])
    .launch()
    .await;
}