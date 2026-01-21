use anyhow::Result;
use gpui::{Bounds, Pixels};

#[cfg(target_os = "macos")]
use cocoa::appkit::{NSWindowStyleMask, NSBackingStoreType};
#[cfg(target_os = "macos")]
use cocoa::base::{id, nil, YES, NO};
#[cfg(target_os = "macos")]
use cocoa::foundation::{NSRect, NSPoint, NSSize, NSString};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl, class};
#[cfg(target_os = "macos")]
use objc::runtime::Class;

/// A floating webview window for embedding web content
pub struct WebViewManager {
    #[cfg(target_os = "macos")]
    floating_window: id,
    #[cfg(target_os = "macos")]
    ns_webview: id,
    current_url: String,
}

impl WebViewManager {
    #[cfg(target_os = "macos")]
    pub fn new(
        _parent_window_ptr: *mut std::ffi::c_void,
        bounds: Bounds<Pixels>,
        url: &str,
    ) -> Result<Self> {
        unsafe {
            let wk_config_class = Class::get("WKWebViewConfiguration")
                .ok_or_else(|| anyhow::anyhow!("WKWebViewConfiguration class not found"))?;
            let wk_webview_class = Class::get("WKWebView")
                .ok_or_else(|| anyhow::anyhow!("WKWebView class not found"))?;

            let screen: id = msg_send![class!(NSScreen), mainScreen];
            let screen_frame: NSRect = msg_send![screen, frame];

            let width: f64 = bounds.size.width.into();
            let height: f64 = bounds.size.height.into();
            let x: f64 = screen_frame.origin.x + 100.0;
            let y: f64 = screen_frame.origin.y + screen_frame.size.height - height - 100.0;

            let window_rect = NSRect {
                origin: NSPoint::new(x, y),
                size: NSSize::new(width, height),
            };

            let style_mask = NSWindowStyleMask::NSTitledWindowMask
                | NSWindowStyleMask::NSClosableWindowMask
                | NSWindowStyleMask::NSMiniaturizableWindowMask
                | NSWindowStyleMask::NSResizableWindowMask
                | NSWindowStyleMask::NSFullSizeContentViewWindowMask;

            let floating_window: id = msg_send![class!(NSWindow), alloc];
            let floating_window: id = msg_send![
                floating_window,
                initWithContentRect:window_rect
                styleMask:style_mask
                backing:NSBackingStoreType::NSBackingStoreBuffered
                defer:NO
            ];

            let _: () = msg_send![floating_window, setTitlebarAppearsTransparent:YES];
            let _: () = msg_send![floating_window, setTitleVisibility:1i64]; // NSWindowTitleHidden
            let _: () = msg_send![floating_window, setLevel:1];
            let _: () = msg_send![floating_window, setOpaque:YES];
            let _: () = msg_send![floating_window, setHasShadow:YES];
            let _: () = msg_send![floating_window, setReleasedWhenClosed:NO];

            let config: id = msg_send![wk_config_class, new];

            let data_store_class = Class::get("WKWebsiteDataStore")
                .ok_or_else(|| anyhow::anyhow!("WKWebsiteDataStore class not found"))?;
            let default_data_store: id = msg_send![data_store_class, defaultDataStore];
            let _: () = msg_send![config, setWebsiteDataStore:default_data_store];

            let webview_frame = NSRect {
                origin: NSPoint::new(0.0, 0.0),
                size: NSSize::new(width, height),
            };

            let webview: id = msg_send![wk_webview_class, alloc];
            let webview: id = msg_send![webview, initWithFrame:webview_frame configuration:config];

            let autoresizing_mask: u64 = 2 | 16;
            let _: () = msg_send![webview, setAutoresizingMask: autoresizing_mask];

            let content_view: id = msg_send![floating_window, contentView];
            let _: () = msg_send![content_view, addSubview:webview];

            let url_string = NSString::alloc(nil).init_str(url);
            let nsurl: id = msg_send![class!(NSURL), URLWithString:url_string];
            let request: id = msg_send![class!(NSURLRequest), requestWithURL:nsurl];
            let _: () = msg_send![webview, loadRequest:request];

            let _: () = msg_send![floating_window, makeKeyAndOrderFront:nil];

            log::info!("Created floating webview window ({}x{})", width, height);

            Ok(Self {
                floating_window,
                ns_webview: webview,
                current_url: url.to_string(),
            })
        }
    }

    #[cfg(target_os = "macos")]
    pub fn navigate(&mut self, url: &str) {
        if self.current_url == url {
            return;
        }

        self.current_url = url.to_string();

        unsafe {
            let url_string = NSString::alloc(nil).init_str(url);
            let nsurl: id = msg_send![class!(NSURL), URLWithString:url_string];
            let request: id = msg_send![class!(NSURLRequest), requestWithURL:nsurl];
            let _: () = msg_send![self.ns_webview, loadRequest:request];
            log::info!("WebView navigated to: {}", url);
        }
    }

    #[cfg(target_os = "macos")]
    pub fn set_hidden(&self, hidden: bool) {
        unsafe {
            if hidden {
                let _: () = msg_send![self.floating_window, orderOut:nil];
            } else {
                let _: () = msg_send![self.floating_window, makeKeyAndOrderFront:nil];
            }
        }
    }

    #[cfg(target_os = "macos")]
    pub fn is_visible(&self) -> bool {
        unsafe {
            let visible: bool = msg_send![self.floating_window, isVisible];
            visible
        }
    }
}

impl Drop for WebViewManager {
    fn drop(&mut self) {
        #[cfg(target_os = "macos")]
        unsafe {
            log::info!("Cleaning up floating webview window");
            let visible: bool = msg_send![self.floating_window, isVisible];
            if visible {
                let _: () = msg_send![self.floating_window, close];
            }
            let _: () = msg_send![self.floating_window, release];
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl WebViewManager {
    pub fn new(_: *mut std::ffi::c_void, _bounds: Bounds<Pixels>, url: &str) -> Result<Self> {
        Ok(Self {
            current_url: url.to_string(),
        })
    }

    pub fn navigate(&mut self, url: &str) {
        self.current_url = url.to_string();
    }

    pub fn set_hidden(&self, _hidden: bool) {}

    pub fn is_visible(&self) -> bool {
        false
    }
}
