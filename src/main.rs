slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    select_backend()?;
    AppWindow::new()?.run()
}

/// Draws the content under a transparent title bar so the native window
/// controls sit on top of the sidebar instead of a separate system bar.
#[cfg(target_os = "macos")]
fn select_backend() -> Result<(), slint::PlatformError> {
    use slint::winit_030::winit::platform::macos::WindowAttributesExtMacOS;

    slint::BackendSelector::new()
        .with_winit_window_attributes_hook(|attributes| {
            attributes
                .with_titlebar_transparent(true)
                .with_fullsize_content_view(true)
                .with_title_hidden(true)
        })
        .select()
}

#[cfg(not(target_os = "macos"))]
fn select_backend() -> Result<(), slint::PlatformError> {
    Ok(())
}
