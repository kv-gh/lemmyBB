use crate::{
    api::{
        comment::report_comment,
        community::get_community,
        extra::{get_last_reply_in_thread, PostOrComment},
        post::{list_posts, report_post},
        site::get_site_data,
        NameOrId,
    },
    env::increased_rate_limit,
    pagination::{PageLimit, Pagination, PAGE_ITEMS},
    routes::{auth, ErrorPage},
};
use anyhow::Error;
use futures::future::join_all;
use rocket::{form::Form, http::CookieJar};
use rocket_dyn_templates::{context, Template};

#[get("/viewforum?<f>&<page>")]
pub async fn view_forum(
    f: i32,
    page: Option<i32>,
    cookies: &CookieJar<'_>,
) -> Result<Template, ErrorPage> {
    let page = page.unwrap_or(1);
    let site_data = get_site_data(cookies).await?;
    let auth = auth(cookies);
    let posts = list_posts(f, PAGE_ITEMS, page, auth.clone()).await?.posts;
    let community = get_community(NameOrId::Id(f), auth.clone()).await?;
    let last_replies = if increased_rate_limit() {
        join_all(
            posts
                .iter()
                .map(|p| get_last_reply_in_thread(p, auth.clone())),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<PostOrComment>, Error>>()?
    } else {
        vec![]
    };

    let limit = PageLimit::Unknown(posts.len());
    let pagination = Pagination::new(page, limit, format!("/viewforum?f={}&", f));
    let ctx = context! { site_data, community, posts, last_replies, pagination };
    Ok(Template::render("viewforum", ctx))
}

#[get("/report?<thread>&<reply>")]
pub async fn report(
    thread: Option<i32>,
    reply: Option<i32>,
    cookies: &CookieJar<'_>,
) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    let action = if let Some(thread) = thread {
        format!("/do_report?thread={}", thread)
    } else if let Some(reply) = reply {
        format!("/do_report?reply={}", reply)
    } else {
        unreachable!()
    };
    let ctx = context! { site_data, action };
    Ok(Template::render("report", ctx))
}

#[derive(FromForm)]
pub struct ReportForm {
    report_text: String,
}

#[post("/do_report?<thread>&<reply>", data = "<form>")]
pub async fn do_report(
    thread: Option<i32>,
    reply: Option<i32>,
    form: Form<ReportForm>,
    cookies: &CookieJar<'_>,
) -> Result<Template, ErrorPage> {
    let site_data = get_site_data(cookies).await?;
    let auth = auth(cookies).unwrap();
    if let Some(thread) = thread {
        report_post(thread, form.report_text.clone(), auth).await?;
    } else if let Some(reply) = reply {
        report_comment(reply, form.report_text.clone(), auth).await?;
    } else {
        unreachable!()
    };
    let message = "Report created";
    Ok(Template::render("message", context! { site_data, message }))
}