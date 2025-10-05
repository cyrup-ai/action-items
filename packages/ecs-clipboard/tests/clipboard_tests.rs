use std::borrow::Cow;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
))]
use action_items_ecs_clipboard::{ClearExtLinux, GetExtLinux, LinuxClipboardKind, SetExtLinux};
use action_items_ecs_clipboard::{Clipboard, Error, ImageData};

const TEXT1: &str = "Hello, world!";
const TEXT2: &str = "Some different text";
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
))]
const TEXT3: &str = "This is for the secondary clipboard";

#[test]
fn get_and_set_text() {
    let mut ctx =
        Clipboard::new().expect("Failed to create clipboard context in get_and_set_text test");
    ctx.set_text(TEXT1.to_owned())
        .expect("Failed to set initial text in clipboard");
    assert_eq!(
        ctx.get_text().expect("Failed to get text from clipboard"),
        TEXT1
    );
    ctx.set_text(TEXT2.to_owned())
        .expect("Failed to set second text in clipboard");
    assert_eq!(
        ctx.get_text()
            .expect("Failed to get updated text from clipboard"),
        TEXT2
    );
}

#[test]
fn clear() {
    let mut ctx = Clipboard::new().expect("Failed to create clipboard context in clear test");
    ctx.set_text(TEXT1.to_owned())
        .expect("Failed to set text before clearing");
    assert_eq!(
        ctx.get_text().expect("Failed to get text before clearing"),
        TEXT1
    );
    ctx.clear().expect("Failed to clear clipboard");
    assert!(matches!(ctx.get_text(), Err(Error::ContentNotAvailable)));
}

#[test]
#[cfg(feature = "image-data")]
fn get_and_set_image() {
    let mut ctx =
        Clipboard::new().expect("Failed to create clipboard context in get_and_set_image test");
    let image = ImageData {
        width: 2,
        height: 2,
        bytes: Cow::from(vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
        ]),
    };
    ctx.set_image(image.clone())
        .expect("Failed to set image in clipboard");
    let image2 = ctx.get_image().expect("Failed to get image from clipboard");
    assert_eq!(image.width, image2.width);
    assert_eq!(image.height, image2.height);
    assert_eq!(image.bytes.len(), image2.bytes.len());
    assert_eq!(image.bytes, image2.bytes);
}

#[test]
#[cfg(all(
    unix,
    not(any(target_os = "macos", target_os = "android", target_os = "emscripten")),
))]
fn linux_specific() {
    let mut ctx =
        Clipboard::new().expect("Failed to create clipboard context in linux_specific test");

    // Test if the primary clipboard is not affected by default
    {
        ctx.set()
            .text(TEXT1.to_owned())
            .expect("Failed to set text in default clipboard");
        assert_eq!(
            ctx.get()
                .text()
                .expect("Failed to get text from default clipboard"),
            TEXT1
        );
        let primary_content = ctx
            .get()
            .clipboard(LinuxClipboardKind::Primary)
            .text()
            .expect_err("Expected primary clipboard to be unavailable but got content");
        assert!(matches!(primary_content, Error::ContentNotAvailable));
    }

    // Test if we can write to the primary clipboard
    {
        ctx.set()
            .clipboard(LinuxClipboardKind::Primary)
            .text(TEXT2.to_owned())
            .expect("Failed to set text in primary clipboard");
        assert_eq!(
            ctx.get()
                .clipboard(LinuxClipboardKind::Primary)
                .text()
                .expect("Failed to get text from primary clipboard"),
            TEXT2
        );
    }

    // Test if clearing works on the primary clipboard
    {
        ctx.clear()
            .clipboard(LinuxClipboardKind::Primary)
            .expect("Failed to clear primary clipboard");
        let primary_content = ctx
            .get()
            .clipboard(LinuxClipboardKind::Primary)
            .text()
            .expect_err(
                "Expected primary clipboard to be unavailable after clearing but got content",
            );
        assert!(matches!(primary_content, Error::ContentNotAvailable));
    }

    // Test if we can write to the secondary clipboard
    {
        ctx.set()
            .clipboard(LinuxClipboardKind::Secondary)
            .text(TEXT3.to_owned())
            .expect("Failed to set text in secondary clipboard");
        assert_eq!(
            ctx.get()
                .clipboard(LinuxClipboardKind::Secondary)
                .text()
                .expect("Failed to get text from secondary clipboard"),
            TEXT3
        );
    }

    // Test if clearing works on the secondary clipboard
    {
        ctx.clear()
            .clipboard(LinuxClipboardKind::Secondary)
            .expect("Failed to clear secondary clipboard");
        let primary_content = ctx
            .get()
            .clipboard(LinuxClipboardKind::Secondary)
            .text()
            .expect_err(
                "Expected secondary clipboard to be unavailable after clearing but got content",
            );
        assert!(matches!(primary_content, Error::ContentNotAvailable));
    }

    // Test waiting for clipboard contents
    {
        let mut ctx =
            Clipboard::new().expect("Failed to create clipboard context for waiting test");
        ctx.set()
            .clipboard(LinuxClipboardKind::Primary)
            .text(TEXT1.to_owned())
            .expect("Failed to set primary clipboard text for waiting test");
        ctx.set()
            .clipboard(LinuxClipboardKind::Secondary)
            .text(TEXT3.to_owned())
            .expect("Failed to set secondary clipboard text for waiting test");

        // Wait for both clipboards to have contents.
        ctx.get()
            .wait()
            .clipboard(LinuxClipboardKind::Primary)
            .text()
            .expect("Failed to wait for primary clipboard content");
        ctx.get()
            .wait()
            .clipboard(LinuxClipboardKind::Secondary)
            .text()
            .expect("Failed to wait for secondary clipboard content");

        // Check if both clipboards have the correct contents.
        assert_eq!(
            TEXT1,
            &ctx.get()
                .clipboard(LinuxClipboardKind::Primary)
                .text()
                .expect("Failed to get primary clipboard text for verification")
        );
        assert_eq!(
            TEXT3,
            &ctx.get()
                .clipboard(LinuxClipboardKind::Secondary)
                .text()
                .expect("Failed to get secondary clipboard text for verification")
        );
    }

    // Test waiting for clipboard contents to be replaced
    {
        let mut ctx =
            Clipboard::new().expect("Failed to create clipboard context for replacement test");

        // Wait for both clipboards to have contents.
        {
            let mut setter = ctx.set().wait();
            let primary = setter.clipboard(LinuxClipboardKind::Primary);
            let secondary = setter.clipboard(LinuxClipboardKind::Secondary);
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(100));
                let mut ctx =
                    Clipboard::new().expect("Failed to create clipboard context in spawned thread");
                ctx.set_text(TEXT1.to_owned())
                    .expect("Failed to set text in spawned thread");
            });
            primary
                .text("initial text".to_owned())
                .expect("Failed to set initial text on primary clipboard");
            secondary
                .text(TEXT3.to_owned())
                .expect("Failed to set text on secondary clipboard");
            assert_eq!(
                TEXT1,
                &ctx.get()
                    .clipboard(LinuxClipboardKind::Clipboard)
                    .text()
                    .expect("Failed to get text from default clipboard after thread update")
            );
            assert_eq!(
                TEXT3,
                &ctx.get()
                    .clipboard(LinuxClipboardKind::Secondary)
                    .text()
                    .expect("Failed to get text from secondary clipboard after thread update")
            );
        }

        let was_replaced = Arc::new(AtomicBool::new(false));

        let setter = thread::spawn({
            let was_replaced = was_replaced.clone();
            move || {
                thread::sleep(Duration::from_millis(100));
                let mut ctx = Clipboard::new()
                    .expect("Failed to create clipboard context in replacement thread");
                ctx.set_text("replacement text".to_owned())
                    .expect("Failed to set replacement text in thread");
                was_replaced.store(true, Ordering::Release);
            }
        });

        ctx.set()
            .wait()
            .text("initial text".to_owned())
            .expect("Failed to set initial text with wait");

        assert!(was_replaced.load(Ordering::Acquire));

        setter.join().expect("Failed to join setter thread");
    }
}

// The cross-platform abstraction should allow any number of clipboards
// to be open at once without issue, as documented under [Clipboard].
#[test]
fn multiple_clipboards_at_once() {
    const THREAD_COUNT: usize = 100;

    let mut handles = Vec::with_capacity(THREAD_COUNT);
    let barrier = Arc::new(std::sync::Barrier::new(THREAD_COUNT));

    for _ in 0..THREAD_COUNT {
        let barrier = barrier.clone();
        handles.push(thread::spawn(move || {
            // As long as the clipboard isn't used multiple times at once, multiple instances
            // are perfectly fine.
            let _ctx = Clipboard::new()
                .expect("Failed to create clipboard context in multi-clipboard thread");

            thread::sleep(Duration::from_millis(10));

            barrier.wait();
        }));
    }

    for thread_handle in handles {
        thread_handle
            .join()
            .expect("Failed to join thread in multi-clipboard test");
    }
}

#[test]
fn clipboard_trait_consistently() {
    fn assert_send_sync<T: Send + Sync + 'static>() {}

    assert_send_sync::<Clipboard>();
    assert!(std::mem::needs_drop::<Clipboard>());
}
