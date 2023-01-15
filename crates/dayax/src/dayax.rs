use axum::extract::State;
use axum::routing::MethodFilter;
use mlua::Lua;
use mlua::UserData;
use std::sync::Arc;
use tracing::debug;

use axum::Router;
use mlua::Function;

use crate::handler::DayaxRequestHandler;
use crate::DayaxRouter;

#[derive(Debug, Clone, Default)]
pub struct Dayax {
    router: DayaxRouter,
}

impl Dayax {
    pub fn new() -> Dayax {
        Default::default()
    }
}

impl From<Dayax> for DayaxRouter {
    fn from(value: Dayax) -> Self {
        value.router
    }
}

macro_rules! gen_dayax_http_verb {
    ( $name:tt , $filter:expr ) => {
        fn $name<'lua, 'this>(
            lua: &'lua Lua,
            this: &'this mut Self,
            (path, callback): (String, Function),
        ) -> mlua::Result<()> {
            this.route(lua, &path, $filter, callback)
        }
    };
}

impl Dayax {
    fn route<'lua>(
        &mut self,
        lua: &'lua Lua,
        path: &str,
        method: MethodFilter,
        callback: Function<'lua>,
    ) -> mlua::Result<()> {
        let info = callback.info();
        let line = info.line_defined;

        let mut temp = Router::new();
        std::mem::swap(&mut temp, &mut self.router);
        let registry_key = lua.create_registry_value(callback)?;
        let registry_key = Arc::new(registry_key);
        let handler = move |State(state): State<_>, req| {
            DayaxRequestHandler::new(registry_key, state).handle(req)
        };
        self.router = temp.route(&path, axum::routing::on(method, handler));

        debug!(?method, path, line, "Loaded request handler");

        Ok(())
    }

    gen_dayax_http_verb!(get, MethodFilter::GET);
    gen_dayax_http_verb!(head, MethodFilter::HEAD);
    gen_dayax_http_verb!(post, MethodFilter::POST);
    gen_dayax_http_verb!(put, MethodFilter::PUT);
    gen_dayax_http_verb!(patch, MethodFilter::PATCH);
    gen_dayax_http_verb!(delete, MethodFilter::DELETE);
    gen_dayax_http_verb!(any, MethodFilter::all());
}

impl UserData for Dayax {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_method_mut("get", Dayax::get);
        methods.add_method_mut("head", Dayax::head);
        methods.add_method_mut("post", Dayax::post);
        methods.add_method_mut("put", Dayax::put);
        methods.add_method_mut("patch", Dayax::patch);
        methods.add_method_mut("delete", Dayax::delete);
        methods.add_method_mut("any", Dayax::any);
    }
}
