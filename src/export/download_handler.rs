use js_sys;
use wasm_bindgen::prelude::*;
use web_sys;

pub struct DownloadHandler;

impl DownloadHandler {
    /// ブラウザでファイルをダウンロード
    pub fn download_image(data: &[u8], format: &str) {
        web_sys::console::log_1(&"Creating blob".into());
        let bag = web_sys::BlobPropertyBag::new();
        bag.set_type(&format!("image/{}", format));
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
            &js_sys::Array::of1(&js_sys::Uint8Array::from(data)),
            &bag,
        )
        .unwrap();
        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
        web_sys::console::log_1(&format!("Blob URL: {}", url).into());
        let document = web_sys::window().unwrap().document().unwrap();
        let a = document
            .create_element("a")
            .unwrap()
            .dyn_into::<web_sys::HtmlElement>()
            .unwrap();
        a.set_attribute("href", &url).unwrap();
        a.set_attribute("download", &format!("exported.{}", format))
            .unwrap();
        a.click();
        web_sys::Url::revoke_object_url(&url).unwrap();
        web_sys::console::log_1(&"Download initiated".into());
    }
}
