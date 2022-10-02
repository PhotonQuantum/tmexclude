#![allow(non_snake_case)]

use cocoa::appkit::{CGFloat, NSView, NSWindow, NSWindowButton};
use cocoa::appkit::NSWindowTitleVisibility;
use cocoa::base::id;
use cocoa::foundation::{NSPoint, NSRect};
use objc::msg_send;
use tauri::{Runtime, Window, WindowEvent};

#[derive(Debug, Copy, Clone)]
pub struct Margin {
    x: CGFloat,
    y: CGFloat,
}

pub trait WindowExt {
    fn set_transparent_titlebar(&self);
    fn set_trafficlights_position(&self, x: CGFloat, y: CGFloat);
}

impl<R: Runtime> WindowExt for Window<R> {
    fn set_transparent_titlebar(&self) {
        unsafe {
            let id = self.ns_window().unwrap() as id;

            id.setTitleVisibility_(NSWindowTitleVisibility::NSWindowTitleHidden);
            id.setTitlebarAppearsTransparent_(cocoa::base::YES);
        }
    }
    fn set_trafficlights_position(&self, x: CGFloat, y: CGFloat) {
        let margin = Margin { x, y };

        self.on_window_event({
            let window = self.clone();
            move |ev| {
                if let WindowEvent::Resized(_) | WindowEvent::Focused(true) = ev {
                    unsafe {
                        let id = window.ns_window().unwrap() as id;
                        update_layout(id, margin);
                    };
                }
            }
        });

        unsafe {
            let id = self.ns_window().unwrap() as id;

            update_layout(id, margin);
        }
    }
}

unsafe fn update_layout(window: impl NSWindow + Copy, margin: Margin) {
    let left = window.standardWindowButton_(NSWindowButton::NSWindowCloseButton);
    let middle = window.standardWindowButton_(NSWindowButton::NSWindowMiniaturizeButton);
    let right = window.standardWindowButton_(NSWindowButton::NSWindowZoomButton);

    let button_width = NSView::frame(left).size.width;
    let button_height = NSView::frame(left).size.height;
    let padding = NSView::frame(middle).origin.x - NSView::frame(left).origin.x - button_width;

    let container = left.superview().superview();
    let mut cbounds = NSView::frame(container);
    cbounds.size.height = button_height + 2. * margin.y;
    cbounds.origin.y = window.frame().size.height - cbounds.size.height;
    container.setFrame(cbounds);

    for (idx, btn) in [left, middle, right].into_iter().enumerate() {
        btn.setFrameOrigin(NSPoint::new(
            margin.x + (button_width + padding) * idx as CGFloat,
            margin.y,
        ));
    }
}

pub trait NSViewExt: Sized {
    unsafe fn setFrame(self, frame: NSRect);
}

impl NSViewExt for id {
    unsafe fn setFrame(self, frame: NSRect) {
        let _: () = msg_send![self, setFrame: frame];
    }
}
