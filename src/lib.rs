#[macro_use]
mod browser;

use anyhow::{anyhow, Result};
use browser::window;
use serde::Deserialize;
use std::{collections::HashMap, rc::Rc, sync::Mutex};
use wasm_bindgen::{prelude::*, JsCast};

const SPRITE_SHEET_INFO: &str = "dropship.json";
const IMAGE_NAME: &str = "dropship.png";
#[derive(Deserialize)]
struct Rect {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

#[derive(Deserialize)]
struct Cell {
    frame: Rect,
}

#[derive(Deserialize)]
struct Sheet {
    frames: HashMap<String, Cell>,
}

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let context = browser::context().expect("unable to get browser context");
    browser::spawn_local(async move {
        let (success_tx, success_rx) = futures::channel::oneshot::channel::<Result<(), JsValue>>();
        let success_tx = Rc::new(Mutex::new(Some(success_tx)));
        let error_tx = Rc::clone(&success_tx);
        let json = browser::fetch_json("dropship.json")
            .await
            .expect(format!("unable to fetch {}", SPRITE_SHEET_INFO).as_str());
        let sheet: Sheet =
            serde_wasm_bindgen::from_value(json).expect("could not deserialize JSON");
        let image = web_sys::HtmlImageElement::new().unwrap();
        let callback = Closure::once(move || {
            if let Some(success_tx) = success_tx.lock().ok().and_then(|mut opt| opt.take()) {
                success_tx
                    .send(Ok(()))
                    .expect("unexpected error sending image load complete");
            }
        });
        let error_callback = Closure::once(move |err| {
            if let Some(error_tx) = error_tx.lock().ok().and_then(|mut opt| opt.take()) {
                error_tx
                    .send(Err(err))
                    .expect("unexpected error sending image load error");
            }
        });
        image.set_onload(Some(callback.as_ref().unchecked_ref()));
        image.set_onerror(Some(error_callback.as_ref().unchecked_ref()));
        image.set_src(IMAGE_NAME);
        success_rx.await;
        let window = window().expect("unable to get window");
        let mut idx = 1;
        let interval_callback = Closure::wrap(Box::new(move || {
            context.clear_rect(0.0, 0.0, 800.0, 600.0);
            let frame = format!("Run ({}).png", idx);
            let sprite = sheet.frames.get(&frame).expect("cell not found");
            context.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                &image,
                sprite.frame.x.into(),
                sprite.frame.y.into(),
                sprite.frame.w.into(),
                sprite.frame.h.into(),
                300.0,
                300.0,
                sprite.frame.w.into(),
                sprite.frame.h.into(),
            );        
            idx += 1;
            if idx > 8 {
                idx = 1;
            }
        }) as Box<dyn FnMut()>);
        window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                interval_callback.as_ref().unchecked_ref(),
                50,
            )
            .expect("unable to set animnation");
        interval_callback.forget();

    });

    Ok(())
}
