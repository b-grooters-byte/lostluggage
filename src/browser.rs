use anyhow::{anyhow, Result};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Document, Window};

const CANVAS_ID: &str = "main_canvas";

macro_rules! log {
    ($( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )*).into());
    }
}

pub fn window() -> Result<Window> {
    web_sys::window().ok_or_else(|| anyhow!("window element not found"))
}

pub fn document() -> Result<Document> {
    window()?
        .document()
        .ok_or_else(|| anyhow!("document node not found"))
}

pub fn canvas() -> Result<web_sys::HtmlCanvasElement> {
    document()?
        .get_element_by_id(CANVAS_ID)
        .ok_or_else(|| anyhow!(format!("canvas element \"{}\"not found", CANVAS_ID)))?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|element| anyhow!("unable to convert {:#?} to HtmlCanvasElement", element))
}

pub fn context() -> Result<web_sys::CanvasRenderingContext2d> {
    canvas()?
        .get_context("2d")
        .map_err(|js_value| anyhow!("error getting canvas 2d context: {:#?}", js_value))?
        .ok_or_else(|| anyhow!("canvas 2d context not found"))?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|element| {
            anyhow!(
                "unable to convert {:#?} to CanvasRenderingContext2d",
                element
            )
        })
}

pub fn spawn_local<F>(future: F)
where
    F: core::future::Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}

pub async fn fetch_with_str(res: &str) -> Result<JsValue> {
    JsFuture::from(window()?.fetch_with_str(res))
        .await
        .map_err(|err| anyhow!("unable to load resource: {:#?}", err))
}

pub async fn fetch_json(path: &str) -> Result<JsValue> {
    let resp_val = fetch_with_str(path).await?;
    let resp: web_sys::Response = resp_val
        .dyn_into()
        .map_err(|element| anyhow!("unable to convert {:#?} to Response", element))?;
    JsFuture::from(
        resp.json()
            .map_err(|err| anyhow!("unable to load JSON from response {:#?}", err))?,
    )
    .await
    .map_err(|err| anyhow!("unable to fetch JSON {:#?}", err))
}
