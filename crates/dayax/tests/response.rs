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
#[trace]
#[tokio::test]
async fn handler_without_return_responds_with_default_response(
    state: DayaxState,
) -> Result<(), Box<dyn Error>> {
    let router = {
        let lua = state.lock().await;
        lua.load(
            r#"
        dayax:get("/", function() 
        end)
        "#,
        )
        .exec()?;
        extract_router(&lua)?
    };

    let response = router
        .with_state(state)
        .oneshot(Request::builder().uri("/").body(Body::empty())?)
        .await?;

    assert_that!(response.status()).is_equal_to(StatusCode::OK);

    Ok(())
}
