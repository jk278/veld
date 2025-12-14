use dioxus::prelude::*;
use std::sync::{Arc, Mutex};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    // Launch the application
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        div {
            id: "app",
            h1 { "Veld - AI Toolkit for Developers" }
            p { "Cross-platform system tray application with AI assistant capabilities" }
            div { id: "features",
                h2 { "Features" }
                ul {
                    li { "System tray integration" }
                    li { "Global keyboard shortcuts" }
                    li { "AI-powered tools" }
                    li { "Context-aware operations" }
                }
            }
            div { id: "status",
                p { "Status: Initializing..." }
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        // Basic test
        assert_eq!(2 + 2, 4);
    }
}
