use crate::{
    api,
    api::{
        site::get_site_data,
        user::{get_captcha, get_person, mark_all_as_read},
        NameOrId,
    },
    routes::{auth, ErrorPage},
};
use rocket::{
    form::Form,
    http::{Cookie, CookieJar},
    response::Redirect,
    Either,
};
use rocket_dyn_templates::{context, Template};

#[get("/login")]
pub async fn login(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    Ok(Template::render("user/login", context!(site_data)))
}

#[derive(FromForm)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[post("/do_login", data = "<form>")]
pub async fn do_login(
    form: Form<LoginForm>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ErrorPage> {
    let jwt = api::user::login(&form.username, &form.password)
        .await?
        .jwt
        .unwrap()
        .into_inner();
    cookies.add(Cookie::new("jwt", jwt));
    Ok(Redirect::to(uri!("/")))
}

#[get("/register")]
pub async fn register(cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    let captcha = get_captcha().await?;
    Ok(Template::render(
        "user/register",
        context!(site_data, captcha),
    ))
}

#[derive(FromForm, Default)]
pub struct RegisterForm {
    pub username: String,
    pub password: String,
    pub password_verify: String,
    pub show_nsfw: bool,
    pub email: Option<String>,
    pub captcha_uuid: Option<String>,
    pub captcha_answer: Option<String>,
    pub honeypot: Option<String>,
    pub application_answer: Option<String>,
    pub refresh_captcha: Option<String>,
}

#[post("/do_register", data = "<form>")]
pub async fn do_register(
    mut form: Form<RegisterForm>,
    cookies: &CookieJar<'_>,
) -> Result<Either<Template, Redirect>, ErrorPage> {
    if form.refresh_captcha.is_some() {
        // user requested new captcha, so reload page
        return Ok(Either::Right(Redirect::to(uri!(register))));
    }

    // empty fields gets parsed into Some(""), convert that to None
    form.captcha_answer = form.captcha_answer.clone().filter(|h| !h.is_empty());
    form.honeypot = form.honeypot.clone().filter(|h| !h.is_empty());
    form.email = form.email.clone().filter(|h| !h.is_empty());
    form.application_answer = form.application_answer.clone().filter(|h| !h.is_empty());

    let res = api::user::register(form.into_inner()).await?;
    let message = if let Some(jwt) = res.jwt {
        cookies.add(Cookie::new("jwt", jwt.into_inner()));
        "Registration successful"
    } else if res.verify_email_sent {
        "Registration successful, confirm your email address"
    } else {
        "Registration successful, wait for admin approval"
    };

    let site = get_site_data(cookies).await?;
    let ctx = context!(site, message);
    Ok(Either::Left(Template::render("message", ctx)))
}

#[get("/logout")]
pub async fn logout(cookies: &CookieJar<'_>) -> Result<Redirect, ErrorPage> {
    // simply delete the cookie
    cookies.remove(Cookie::named("jwt"));
    Ok(Redirect::to(uri!("/")))
}

#[post("/mark_all_notifications_read")]
pub async fn mark_all_notifications_read(cookies: &CookieJar<'_>) -> Result<Redirect, ErrorPage> {
    mark_all_as_read(auth(cookies).unwrap()).await?;
    Ok(Redirect::to(uri!("/")))
}

#[get("/view_profile?<u>")]
pub async fn view_profile(u: i32, cookies: &CookieJar<'_>) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    let person = get_person(NameOrId::Id(u), auth(cookies)).await?;
    let ctx = context!(site_data, person);
    Ok(Template::render("user/view_profile", ctx))
}
