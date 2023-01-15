use std::error::Error;
use std::sync::Arc;

use ::dayax::Dayax;
use ::dayax::DayaxRouter;
use ::dayax::DayaxState;
use axum::body::Body;
use http::Request;
use http::StatusCode;
use mlua::Lua;
use rstest::*;
use speculoos::*;
use tokio::sync::Mutex;
use tower::util::ServiceExt;

#[fixture]
fn dayax() -> Dayax {
    Dayax::new()
}

#[fixture]
fn lua(dayax: Dayax) -> Lua {
    let lua = Lua::new();
    lua.globals().set("dayax", dayax).unwrap();
    lua
}
#[fixture]
fn state(lua: Lua) -> DayaxState {
    Arc::new(Mutex::new(lua))
}

fn extract_router(lua: &Lua) -> mlua::Result<DayaxRouter> {
    let dayax: Dayax = lua.globals().get("dayax")?;
    Ok(dayax.into())
}

#[rstest]
#[case::empty(
    r#"
    dayax:get("/", function() 
    end)"#,
    StatusCode::OK,
    b"",
    vec![]
)]
#[case::nil(
    r#"
    dayax:get("/", function() 
    end)"#,
    StatusCode::OK,
    b"",
    vec![]
)]
#[case::string(
    r#"
    dayax:get("/", function()
        return "Hello world"
    end)"#,
    StatusCode::OK,
    b"Hello world",
    vec![("Content-Type", "text/plain; charset=utf-8")]
)]
#[case::table_body_string(
    r#"
    dayax:get("/", function()
        return { body = "hello world" }
    end)"#,
    StatusCode::OK,
    b"hello world",
    vec![("Content-Type", "text/plain; charset=utf-8")]
)]
#[case::table_body_json_array(
    r#"
    dayax:get("/", function()
        return { body = { 1,2,3 } }
    end)"#,
    StatusCode::OK,
    br#"[1,2,3]"#,
    vec![("Content-Type", "application/json")]
)]
#[case::table_body_json_object(
    r#"
    dayax:get("/", function()
        return { body = { hello = "world" } }
    end)"#,
    StatusCode::OK,
    br#"{"hello":"world"}"#,
    vec![("Content-Type", "application/json")]
)]
#[case::table_statuscode(
    r#"
    dayax:get("/", function()
        return { statusCode = 418 }
    end)"#,
    StatusCode::IM_A_TEAPOT,
    b"",
    vec![]
)]
#[case::table_redirect(
    r#"
    dayax:get("/", function()
        return { redirect = '/redirect' }
    end)"#,
    StatusCode::TEMPORARY_REDIRECT,
    b"",
    vec![("location", "/redirect")]
)]
#[case::error(
    r#"
    dayax:get("/", function()
        error("My error")
    end)"#,
    StatusCode::INTERNAL_SERVER_ERROR,
    b"Internal Server Error",
    vec![]
)]
#[trace]
#[tokio::test]
async fn handler_responds_with_expected_response(
    state: DayaxState,
    #[case] lua_code: &str,
    #[case] expected_status_code: StatusCode,
    #[case] expected_body: &[u8],
    #[case] expected_headers: Vec<(&str, &str)>,
) -> Result<(), Box<dyn Error>> {
    let router = {
        let lua = state.lock().await;
        lua.load(lua_code).exec()?;
        extract_router(&lua)?
    };

    let response = router
        .with_state(state)
        .oneshot(Request::builder().uri("/").body(Body::empty())?)
        .await?;

    assert_that!(response.status()).is_equal_to(expected_status_code);

    let headers = response.headers();
    for (expected_name, expected_value) in expected_headers {
        let value = headers.get(expected_name).expect("Missing header");
        assert_that!(value.to_str()?).is_equal_to(expected_value);
    }

    let body = hyper::body::to_bytes(response.into_body()).await?;
    assert_that(&&body[..]).is_equal_to(expected_body);

    Ok(())
}
