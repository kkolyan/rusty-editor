use crate::{
    inspector::SenderHelper,
    scene::commands::{camera::SetExposureCommand, SceneCommand},
};
use rg3d::gui::message::FieldKind;
use rg3d::{
    core::pool::Handle,
    gui::message::PropertyChanged,
    scene::{camera::Exposure, node::Node},
};

pub fn handle_camera_property_changed(
    args: &PropertyChanged,
    node_handle: Handle<Node>,
    helper: &SenderHelper,
) {
    if let FieldKind::Object(ref value) = args.value {
        match args.name.as_ref() {
            "exposure" => helper.do_scene_command(SceneCommand::SetExposure(
                SetExposureCommand::new(node_handle, *value.cast_value::<Exposure>().unwrap()),
            )),
            _ => println!("Unhandled property of Camera: {:?}", args),
        }
    }
}