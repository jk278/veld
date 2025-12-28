//! Application routing definitions
//! 使用类型安全的路由系统组织多页面应用

use crate::components::{
  about::About, home::Home, layout::AppLayout, result_viewer::ResultViewer, settings::Settings,
};
use dioxus::prelude::*;

#[derive(Clone, Routable)]
#[rustfmt::skip]
pub enum Route {
  #[layout(AppLayout)]
  #[route("/")]
  Home,
  #[route("/settings")]
  Settings,
  #[route("/result/:session_id")]
  ResultViewer { session_id: String },
  #[route("/about")]
  About,
}
