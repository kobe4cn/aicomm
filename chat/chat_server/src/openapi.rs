use axum::Router;
use core_lib::{Chat, ChatType, ChatUser, Message, User, WorkSpace};
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::{
    handlers::*,
    models::{CreateChat, CreateUser, SigninUser},
    ErrorOutput,
};
use crate::{
    models::{CreateAgent, UpdateAgent},
    AppState,
};
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_scalar::{Scalar, Servable as _};
use utoipa_swagger_ui::SwaggerUi;
pub(crate) trait OpenApiRouter {
    fn openapi(self) -> Self;
}

#[derive(OpenApi)]
#[openapi(
    paths(
        signup_handler,
        signin_handler,
        list_chat_handler,
        create_chat_handler,
        get_chat_handler,
        list_messages_handler,
        list_agents_handler,
        create_agent_handler,
        update_agent_handler,

    ),
        components(schemas( User,Chat,ChatType,ChatUser,Message,WorkSpace,SigninUser,CreateUser,CreateChat,AuthOutput,ErrorOutput,CreateAgent,UpdateAgent)),
        modifiers(&SecurityAddon),
        tags(
            (name = "todo", description = "Todo items management API")
        )
    )]
pub(crate) struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "token",
                SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build()),
            )
        }
    }
}

impl OpenApiRouter for Router<AppState> {
    fn openapi(self) -> Self {
        self.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
            .merge(Redoc::with_url("/redoc", ApiDoc::openapi()))
            // There is no need to create `RapiDoc::with_openapi` because the OpenApi is served
            // via SwaggerUi instead we only make rapidoc to point to the existing doc.
            .merge(RapiDoc::new("/api-docs/openapi.json").path("/rapidoc"))
            // Alternative to above
            // .merge(RapiDoc::with_openapi("/api-docs/openapi2.json", api).path("/rapidoc"))
            .merge(Scalar::with_url("/scalar", ApiDoc::openapi()))
    }
}
