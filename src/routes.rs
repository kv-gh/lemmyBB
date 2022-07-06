use crate::{
    api::{create_comment, get_post, get_site, list_posts, login, CLIENT},
    error::ErrorPage,
};
use reqwest::header::HeaderName;
use rocket::{
    form::Form,
    http::{Cookie, CookieJar},
    response::Redirect,
};
use rocket_dyn_templates::{context, Template};
use url::Url;

#[get("/")]
pub async fn view_forum() -> Result<Template, ErrorPage> {
    let site = get_site().await?.site_view.unwrap();
    let posts = list_posts().await?.posts;
    let ctx = context! { site, posts };
    Ok(Template::render("viewforum", ctx))
}

#[get("/viewtopic?<t>")]
pub async fn view_topic(t: i32) -> Result<Template, ErrorPage> {
    let site = get_site().await?.site_view.unwrap();
    let mut post = get_post(t).await?;

    // show oldest comments first
    post.comments
        .sort_by(|a, b| a.comment.published.cmp(&b.comment.published));

    // simply ignore deleted/removed comments
    post.comments = post
        .comments
        .into_iter()
        .filter(|c| !c.comment.deleted && !c.comment.removed)
        .collect();

    // determine if post.url should be rendered as <img> or <a href>
    let mut is_image_url = false;
    if let Some(ref url) = post.post_view.post.url {
        // TODO: use HEAD request once that is supported by pictrs/lemmy
        let image = CLIENT.get::<Url>(url.clone().into()).send().await?;
        let content_type = &image.headers()[HeaderName::from_static("content-type")];
        is_image_url = content_type.to_str()?.starts_with("image/");
    }

    let ctx = context! { site, post, is_image_url };
    Ok(Template::render("viewtopic", ctx))
}

#[get("/login")]
pub async fn login_page() -> Result<Template, ErrorPage> {
    let site = get_site().await?.site_view.unwrap();
    Ok(Template::render("login", context!(site)))
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
    let jwt = login(&form.username, &form.password)
        .await?
        .jwt
        .unwrap()
        .into_inner();
    cookies.add(Cookie::new("jwt", jwt));
    Ok(Redirect::to(uri!(view_forum)))
}

#[get("/posting?<t>")]
pub async fn posting(t: i32) -> Result<Template, ErrorPage> {
    let post = get_post(t).await?;
    let site = get_site().await?.site_view.unwrap();
    Ok(Template::render("posting", context!(site, post)))
}

#[derive(FromForm)]
pub struct PostForm {
    message: String,
}

#[post("/do_post?<t>", data = "<form>")]
pub async fn do_post(
    t: i32,
    form: Form<PostForm>,
    cookies: &CookieJar<'_>,
) -> Result<Redirect, ErrorPage> {
    create_comment(t, form.message.clone(), cookies.get("jwt").unwrap().value()).await?;
    Ok(Redirect::to(uri!(view_topic(t))))
}
