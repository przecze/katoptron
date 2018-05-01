extern crate dbus;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub fn show(stop: Arc<AtomicBool>) {
	let factory = dbus::tree::Factory::new_fn::<()>();

	let new_title_signal = Arc::new(factory.signal("NewTitle", ()));
	let new_icon_signal = Arc::new(factory.signal("NewIcon", ()));
	let new_attention_icon_signal = Arc::new(factory.signal("NewAttentionIcon", ()));
	let new_overlay_icon_signal = Arc::new(factory.signal("NewOverlayIcon", ()));
	let new_tooltip_signal = Arc::new(factory.signal("NewTooltip", ()));
	let new_status_signal = Arc::new(factory.signal("NewStatus", ()).sarg::<&str, _>("status"));

	//IconPixmap(x: i32, y: i32, data: &[u8]), OverlayIconPixmap, AttentionIconPixmap, AttentionMovieName(name: &str), ToolTip(icon_name: &str, icon_data: (i32, i32, &[u8]), title: &str, description: &str)

	let tree = factory.tree(())
		.add(factory.object_path("/StatusNotifierItem", ()).introspectable()
			.add(factory.interface("org.kde.StatusNotifierItem", ())
				.add_p(factory.property::<&str, _>("Category", ()).emits_changed(dbus::tree::EmitsChangedSignal::Const).on_get(|response, _| {
					response.append("Communications");
					Ok(())
				}))
				.add_p(factory.property::<&str, _>("AttentionIconName", ()).emits_changed(dbus::tree::EmitsChangedSignal::Const).on_get(|response, _| {
					response.append("dialog-error");
					Ok(())
				}))
				.add_p(factory.property::<&str, _>("IconName", ()).emits_changed(dbus::tree::EmitsChangedSignal::Const).on_get(|response, _| {
					response.append("dialog-information");
					Ok(())
				}))
				.add_p(factory.property::<&str, _>("OverlayIconName", ()).emits_changed(dbus::tree::EmitsChangedSignal::Const).on_get(|response, _| {
					response.append("dialog-error");
					Ok(())
				}))
				.add_p(factory.property::<&str, _>("Id", ()).emits_changed(dbus::tree::EmitsChangedSignal::Const).on_get(|response, _| {
					response.append("notifs_recv");
					Ok(())
				}))
				.add_p(factory.property::<&str, _>("Title", ()).emits_changed(dbus::tree::EmitsChangedSignal::Const).on_get(|response, _| {
					response.append("VM Notifications");
					Ok(())
				}))
				.add_p(factory.property::<&str, _>("Status", ()).emits_changed(dbus::tree::EmitsChangedSignal::Invalidates).access(dbus::tree::Access::Read).on_get(|response, _| {
					response.append("Active"); //NeedsAttention
					Ok(())
				}))
				.add_p(factory.property::<u32, _>("WindowId", ()).emits_changed(dbus::tree::EmitsChangedSignal::Const).on_get(|response, _| {
					response.append(0u32);
					Ok(())
				}))
				.add_p(factory.property::<bool, _>("ItemIsMenu", ()).emits_changed(dbus::tree::EmitsChangedSignal::Const).on_get(|response, _| {
					response.append(false);
					Ok(())
				}))
				.add_p(factory.property::<&str, _>("Menu", ()).emits_changed(dbus::tree::EmitsChangedSignal::Const).on_get(|response, _| {
					response.append("");
					Ok(())
				}))
				.add_m(factory.method("Activate", (), |call| {
					if let (Some(x), Some(y)) = call.msg.get2::<i32, i32>() {
						println!("Left-clicked at ({x}, {y})", x=x, y=y);
					};
					Ok(vec![call.msg.method_return()])
				}).inarg::<i32, _>("x").inarg::<i32, _>("y"))
				.add_m(factory.method("ContextMenu", (), |call| {
					if let (Some(x), Some(y)) = call.msg.get2::<i32, i32>() {
						println!("Right-clicked at ({x}, {y})", x=x, y=y);
					};
					Ok(vec![call.msg.method_return()])
				}).inarg::<i32, _>("x").inarg::<i32, _>("y"))
				.add_m(factory.method("SecondaryActivate", (), |call| {
					if let (Some(x), Some(y)) = call.msg.get2::<i32, i32>() {
						println!("Middle-clicked at ({x}, {y})", x=x, y=y);
					};
					Ok(vec![call.msg.method_return()])
				}).inarg::<i32, _>("x").inarg::<i32, _>("y"))
				.add_m(factory.method("Scroll", (), |call| {
					if let (Some(delta), Some(orientation)) = call.msg.get2::<i32, &str>() {
						println!("Scrolled {orientation}ly by {delta}", delta=delta, orientation=orientation);
					};
					Ok(vec![call.msg.method_return()])
				}).inarg::<i32, _>("delta").inarg::<&str, _>("orientation"))
				.add_s(new_title_signal)
				.add_s(new_icon_signal)
				.add_s(new_attention_icon_signal)
				.add_s(new_overlay_icon_signal)
				.add_s(new_tooltip_signal)
				.add_s(new_status_signal)
			)
		);

	let dbus_conn = dbus::Connection::get_private(dbus::BusType::Session).expect("couldn't connect to D-Bus");
	println!("{}", dbus_conn.unique_name());

	tree.set_registered(&dbus_conn, true).expect("couldn't register D-Bus paths");
	dbus_conn.add_handler(tree);

	let register = dbus::Message::new_method_call("org.kde.StatusNotifierWatcher", "/StatusNotifierWatcher", "org.kde.StatusNotifierWatcher", "RegisterStatusNotifierItem").expect("couldn't create message");
	let register = register.append(dbus_conn.unique_name());
	match dbus_conn.send_with_reply_and_block(register, 250) {
		Ok(_) => {
			println!("Registered!");
		},
		Err(err) => {
			println!("fuck: {}: {}", err.name().expect("b"), err.message().expect("c"));
		},
	}


	while !stop.load(Ordering::SeqCst) {
		for _ in dbus_conn.incoming(50) {}
	}
}