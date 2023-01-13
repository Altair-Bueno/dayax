use crate::handler::HandlerRequest;
use crate::runtime::get_registry_request_key;
use axum::extract::State;
use mlua::Lua;
use mlua::UserData;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use axum::Router;
use mlua::Function;

#[derive(Debug, Clone, Default)]
pub struct Dayax {
    pub router: Router<Arc<Mutex<Lua>>>,
}

macro_rules! gen_dayax_http_verb {
    ( $method_name:tt , $verb:tt ) => {
        fn $method_name<'lua, 'this>(
            lua: &'lua Lua,
            this: &'this mut Self,
            (path, callback): (String, Function),
        ) -> mlua::Result<()> {
            Dayax::route(lua, this, ($verb.into(), path, callback))
        }
    };
}

impl Dayax {
    pub fn new() -> Dayax {
        Default::default()
    }
}

impl Dayax {
    fn route<'lua, 'this>(
        lua: &'lua Lua,
        this: &'this mut Self,
        (method, path, callback): (String, String, Function),
    ) -> mlua::Result<()> {
        let info = callback.info();
        let line = info.line_defined;
        let method = method.to_uppercase();

        let mut temp = Router::new();
        std::mem::swap(&mut temp, &mut this.router);
        let key = get_registry_request_key(&method, &path);
        let registry_key = lua.create_registry_value(callback)?;
        let registry_key = Arc::new(registry_key);
        let handler = move |State(lua_mutex): State<Arc<_>>, req: HandlerRequest| async move {
            crate::handler::request_handler(&lua_mutex, registry_key.clone(), req).await
        };
        let method_router = match method.as_str() {
            "GET" => axum::routing::get(handler),
            "POST" => axum::routing::post(handler),
            "PUT" => axum::routing::put(handler),
            "PATCH" => axum::routing::patch(handler),
            "DELETE" => axum::routing::delete(handler),
            _ => axum::routing::any(handler),
        };
        this.router = temp.route(&path, method_router);

        debug!(method, path, line, key, "Loaded request handler");

        Ok(())
    }

    gen_dayax_http_verb!(get, "GET");
    gen_dayax_http_verb!(post, "POST");
    gen_dayax_http_verb!(put, "PUT");
    gen_dayax_http_verb!(patch, "PATCH");
    gen_dayax_http_verb!(delete, "DELETE");
}

impl UserData for Dayax {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("route", Dayax::route);
        methods.add_method_mut("get", Dayax::get);
        methods.add_method_mut("post", Dayax::post);
        methods.add_method_mut("put", Dayax::put);
        methods.add_method_mut("patch", Dayax::patch);
        methods.add_method_mut("delete", Dayax::delete);
    }
}
