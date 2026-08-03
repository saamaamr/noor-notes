use noor_windowing::{NativeWindowId, WindowController, X11WindowController};
use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt, CreateWindowAux, WindowClass};

fn atom(connection: &impl Connection, name: &[u8]) -> Atom {
    connection
        .intern_atom(false, name)
        .unwrap()
        .reply()
        .unwrap()
        .atom
}

#[tokio::test]
async fn x11_controller_sets_above_workspace_and_opacity_properties() {
    let (connection, screen_number) = x11rb::connect(None).unwrap();
    let screen = &connection.setup().roots[screen_number];
    let window = connection.generate_id().unwrap();
    connection
        .create_window(
            COPY_DEPTH_FROM_PARENT,
            window,
            screen.root,
            0,
            0,
            320,
            240,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new(),
        )
        .unwrap();
    connection.flush().unwrap();
    let controller = X11WindowController::connect().unwrap();
    let native = NativeWindowId::X11(window);

    controller.set_above(native.clone(), true).await.unwrap();
    controller
        .set_all_workspaces(native.clone(), true)
        .await
        .unwrap();
    controller.set_opacity(native.clone(), 0.75).await.unwrap();

    let state = connection
        .get_property(
            false,
            window,
            atom(&connection, b"_NET_WM_STATE"),
            AtomEnum::ATOM,
            0,
            u32::MAX,
        )
        .unwrap()
        .reply()
        .unwrap();
    let above = atom(&connection, b"_NET_WM_STATE_ABOVE");
    assert!(state.value32().unwrap().any(|value| value == above));
    let desktop = connection
        .get_property(
            false,
            window,
            atom(&connection, b"_NET_WM_DESKTOP"),
            AtomEnum::CARDINAL,
            0,
            1,
        )
        .unwrap()
        .reply()
        .unwrap();
    assert_eq!(desktop.value32().unwrap().next(), Some(u32::MAX));
    let opacity = connection
        .get_property(
            false,
            window,
            atom(&connection, b"_NET_WM_WINDOW_OPACITY"),
            AtomEnum::CARDINAL,
            0,
            1,
        )
        .unwrap()
        .reply()
        .unwrap();
    assert_eq!(
        opacity.value32().unwrap().next(),
        Some((0.75 * f64::from(u32::MAX)).round() as u32)
    );

    controller.set_above(native.clone(), false).await.unwrap();
    let state = connection
        .get_property(
            false,
            window,
            atom(&connection, b"_NET_WM_STATE"),
            AtomEnum::ATOM,
            0,
            u32::MAX,
        )
        .unwrap()
        .reply()
        .unwrap();
    assert!(!state.value32().unwrap().any(|value| value == above));
}
