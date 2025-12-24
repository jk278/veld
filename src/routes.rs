//! Application routing definitions
//! 使用类型安全的路由系统组织多页面应用

use dioxus::prelude::*;
use crate::components::{home::Home, settings::Settings, result_viewer::ResultViewer, about::About, layout::AppLayout, ai_config::AiConfig};

#[derive(Clone, Routable)]
#[rustfmt::skip]
pub enum Route {
    #[layout(AppLayout)]
    #[route("/")]
    Home,
    #[route("/settings")]
    Settings,
    #[route("/ai-config")]
    AiConfig,
    #[route("/result/:session_id")]
    ResultViewer { session_id: String },
    #[route("/about")]
    About,
}

