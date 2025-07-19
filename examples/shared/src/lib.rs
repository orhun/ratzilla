use web_sys::wasm_bindgen::JsValue;


pub mod backend;
pub mod fps;
pub mod utils;




/// Available backend types
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum BackendType {
    #[default]
    Dom,
    Canvas,
    WebGl2,
}

impl BackendType {
    /// Get the backend type from a string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "dom" => Some(BackendType::Dom),
            "canvas" => Some(BackendType::Canvas),
            "webgl2" => Some(BackendType::WebGl2),
            _ => None,
        }
    }

    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            BackendType::Dom => "dom",
            BackendType::Canvas => "canvas",
            BackendType::WebGl2 => "webgl2",
        }
    }

    /// Get a human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            BackendType::Dom => "DOM",
            BackendType::Canvas => "Canvas",
            BackendType::WebGl2 => "WebGL2",
        }
    }
}




/// Inject HTML footer with backend switching links
fn inject_backend_footer(current_backend: BackendType) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;

    // Remove existing footer if present
    if let Some(existing) = document.get_element_by_id("ratzilla-backend-footer") {
        existing.remove();
    }

    // Create footer element
    let footer = document.create_element("div")?;
    footer.set_id("ratzilla-backend-footer");

    // Set footer styles
    footer.set_attribute(
        "style",
        "position: fixed; bottom: 0; left: 0; right: 0; \
         background: rgba(0,0,0,0.8); color: white; \
         padding: 8px 16px; font-family: monospace; font-size: 12px; \
         display: flex; justify-content: center; gap: 16px; \
         border-top: 1px solid #333; z-index: 1000;",
    )?;

    // Get current URL without backend param - use relative URL to avoid protocol issues
    let location = window.location();
    let base_url = location.pathname().unwrap_or_default();

    let backends = [BackendType::Dom, BackendType::Canvas, BackendType::WebGl2];
    let mut links = Vec::new();

    for backend in backends {
        let is_current = backend == current_backend;
        let style = if is_current {
            "color: #4ade80; font-weight: bold; text-decoration: none;"
        } else {
            "color: #94a3b8; text-decoration: none; cursor: pointer;"
        };

        let link = if is_current {
            format!(
                "<span style=\"{}\">● {}</span>",
                style,
                backend.display_name()
            )
        } else {
            format!(
                "<a href=\"{}?backend={}\" style=\"{}\">{}</a>",
                base_url,
                backend.as_str(),
                style,
                backend.display_name()
            )
        };

        links.push(link);
    }

    let footer_html = format!(
        "<span style=\"color: #64748b;\">Backend:</span> {} | \
         <span style=\"color: #64748b;\">FPS:</span> \
         <span id=\"ratzilla-fps\" style=\"color: #4ade80; font-weight: bold;\">--</span>",
        links.join(" | ")
    );

    footer.set_inner_html(&footer_html);

    // Append to body
    let body = document.body().ok_or("No body")?;
    body.append_child(&footer)?;

    Ok(())
}

