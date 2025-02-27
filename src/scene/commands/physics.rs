use crate::{
    command::Command,
    physics::{Collider, Joint, RigidBody},
    scene::commands::SceneContext,
    Physics,
};
use rg3d::{
    core::{
        algebra::{UnitQuaternion, Vector3},
        pool::{ErasedHandle, Handle, Ticket},
    },
    physics3d::desc::{ColliderShapeDesc, JointParamsDesc},
    scene::node::Node,
};

#[derive(Debug)]
pub struct AddJointCommand {
    ticket: Option<Ticket<Joint>>,
    handle: Handle<Joint>,
    joint: Option<Joint>,
}

impl AddJointCommand {
    pub fn new(node: Joint) -> Self {
        Self {
            ticket: None,
            handle: Default::default(),
            joint: Some(node),
        }
    }
}

impl Command for AddJointCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Add Joint".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        match self.ticket.take() {
            None => {
                self.handle = context
                    .editor_scene
                    .physics
                    .joints
                    .spawn(self.joint.take().unwrap());
            }
            Some(ticket) => {
                let handle = context
                    .editor_scene
                    .physics
                    .joints
                    .put_back(ticket, self.joint.take().unwrap());
                assert_eq!(handle, self.handle);
            }
        }
    }

    fn revert(&mut self, context: &mut SceneContext) {
        let (ticket, node) = context
            .editor_scene
            .physics
            .joints
            .take_reserve(self.handle);
        self.ticket = Some(ticket);
        self.joint = Some(node);
    }

    fn finalize(&mut self, context: &mut SceneContext) {
        if let Some(ticket) = self.ticket.take() {
            context.editor_scene.physics.joints.forget_ticket(ticket)
        }
    }
}

#[derive(Debug)]
pub struct DeleteJointCommand {
    handle: Handle<Joint>,
    ticket: Option<Ticket<Joint>>,
    node: Option<Joint>,
}

impl DeleteJointCommand {
    pub fn new(handle: Handle<Joint>) -> Self {
        Self {
            handle,
            ticket: None,
            node: None,
        }
    }
}

impl Command for DeleteJointCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Delete Joint".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        let (ticket, node) = context
            .editor_scene
            .physics
            .joints
            .take_reserve(self.handle);
        self.node = Some(node);
        self.ticket = Some(ticket);
    }

    fn revert(&mut self, context: &mut SceneContext) {
        self.handle = context
            .editor_scene
            .physics
            .joints
            .put_back(self.ticket.take().unwrap(), self.node.take().unwrap());
    }

    fn finalize(&mut self, context: &mut SceneContext) {
        if let Some(ticket) = self.ticket.take() {
            context.editor_scene.physics.joints.forget_ticket(ticket)
        }
    }
}

#[derive(Debug)]
pub struct SetBodyCommand {
    node: Handle<Node>,
    ticket: Option<Ticket<RigidBody>>,
    handle: Handle<RigidBody>,
    body: Option<RigidBody>,
}

impl SetBodyCommand {
    pub fn new(node: Handle<Node>, body: RigidBody) -> Self {
        Self {
            node,
            ticket: None,
            handle: Default::default(),
            body: Some(body),
        }
    }
}

impl Command for SetBodyCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Set Node Body".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        match self.ticket.take() {
            None => {
                self.handle = context
                    .editor_scene
                    .physics
                    .bodies
                    .spawn(self.body.take().unwrap());
            }
            Some(ticket) => {
                context
                    .editor_scene
                    .physics
                    .bodies
                    .put_back(ticket, self.body.take().unwrap());
            }
        }
        context
            .editor_scene
            .physics
            .binder
            .insert(self.node, self.handle);
    }

    fn revert(&mut self, context: &mut SceneContext) {
        let (ticket, node) = context
            .editor_scene
            .physics
            .bodies
            .take_reserve(self.handle);
        self.ticket = Some(ticket);
        self.body = Some(node);
        context
            .editor_scene
            .physics
            .binder
            .remove_by_key(&self.node);
    }

    fn finalize(&mut self, context: &mut SceneContext) {
        if let Some(ticket) = self.ticket.take() {
            context.editor_scene.physics.bodies.forget_ticket(ticket);
            context
                .editor_scene
                .physics
                .binder
                .remove_by_key(&self.node);
        }
    }
}

#[derive(Debug)]
pub struct SetColliderCommand {
    body: Handle<RigidBody>,
    ticket: Option<Ticket<Collider>>,
    handle: Handle<Collider>,
    collider: Option<Collider>,
}

impl SetColliderCommand {
    pub fn new(body: Handle<RigidBody>, collider: Collider) -> Self {
        Self {
            body,
            ticket: None,
            handle: Default::default(),
            collider: Some(collider),
        }
    }
}

impl Command for SetColliderCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Set Collider".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        match self.ticket.take() {
            None => {
                self.handle = context
                    .editor_scene
                    .physics
                    .colliders
                    .spawn(self.collider.take().unwrap());
            }
            Some(ticket) => {
                context
                    .editor_scene
                    .physics
                    .colliders
                    .put_back(ticket, self.collider.take().unwrap());
            }
        }
        context.editor_scene.physics.colliders[self.handle].parent = self.body.into();
        context.editor_scene.physics.bodies[self.body]
            .colliders
            .push(self.handle.into());
    }

    fn revert(&mut self, context: &mut SceneContext) {
        let (ticket, mut collider) = context
            .editor_scene
            .physics
            .colliders
            .take_reserve(self.handle);
        collider.parent = Default::default();
        self.ticket = Some(ticket);
        self.collider = Some(collider);

        let body = &mut context.editor_scene.physics.bodies[self.body];
        body.colliders.remove(
            body.colliders
                .iter()
                .position(|&c| c == ErasedHandle::from(self.handle))
                .unwrap(),
        );
    }

    fn finalize(&mut self, context: &mut SceneContext) {
        if let Some(ticket) = self.ticket.take() {
            context.editor_scene.physics.colliders.forget_ticket(ticket);
        }
    }
}

#[derive(Debug)]
pub struct DeleteBodyCommand {
    handle: Handle<RigidBody>,
    ticket: Option<Ticket<RigidBody>>,
    body: Option<RigidBody>,
    node: Handle<Node>,
}

impl DeleteBodyCommand {
    pub fn new(handle: Handle<RigidBody>) -> Self {
        Self {
            handle,
            ticket: None,
            body: None,
            node: Handle::NONE,
        }
    }
}

impl Command for DeleteBodyCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Delete Body".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        let (ticket, node) = context
            .editor_scene
            .physics
            .bodies
            .take_reserve(self.handle);
        self.body = Some(node);
        self.ticket = Some(ticket);
        self.node = context.editor_scene.physics.unbind_by_body(self.handle);
    }

    fn revert(&mut self, context: &mut SceneContext) {
        self.handle = context
            .editor_scene
            .physics
            .bodies
            .put_back(self.ticket.take().unwrap(), self.body.take().unwrap());
        context
            .editor_scene
            .physics
            .binder
            .insert(self.node, self.handle);
    }

    fn finalize(&mut self, context: &mut SceneContext) {
        if let Some(ticket) = self.ticket.take() {
            context.editor_scene.physics.bodies.forget_ticket(ticket)
        }
    }
}

#[derive(Debug)]
pub struct DeleteColliderCommand {
    handle: Handle<Collider>,
    ticket: Option<Ticket<Collider>>,
    collider: Option<Collider>,
    body: Handle<RigidBody>,
}

impl DeleteColliderCommand {
    pub fn new(handle: Handle<Collider>) -> Self {
        Self {
            handle,
            ticket: None,
            collider: None,
            body: Handle::NONE,
        }
    }
}

impl Command for DeleteColliderCommand {
    fn name(&mut self, _context: &SceneContext) -> String {
        "Delete Collider".to_owned()
    }

    fn execute(&mut self, context: &mut SceneContext) {
        let (ticket, collider) = context
            .editor_scene
            .physics
            .colliders
            .take_reserve(self.handle);
        self.body = collider.parent.into();
        self.collider = Some(collider);
        self.ticket = Some(ticket);

        let body = &mut context.editor_scene.physics.bodies[self.body];
        body.colliders.remove(
            body.colliders
                .iter()
                .position(|&c| c == ErasedHandle::from(self.handle))
                .unwrap(),
        );
    }

    fn revert(&mut self, context: &mut SceneContext) {
        self.handle = context
            .editor_scene
            .physics
            .colliders
            .put_back(self.ticket.take().unwrap(), self.collider.take().unwrap());

        let body = &mut context.editor_scene.physics.bodies[self.body];
        body.colliders.push(self.handle.into());
    }

    fn finalize(&mut self, context: &mut SceneContext) {
        if let Some(ticket) = self.ticket.take() {
            context.editor_scene.physics.colliders.forget_ticket(ticket)
        }
    }
}

macro_rules! define_physics_command {
    ($name:ident($human_readable_name:expr, $handle_type:ty, $value_type:ty) where fn swap($self:ident, $physics:ident) $apply_method:block ) => {
        #[derive(Debug)]
        pub struct $name {
            handle: Handle<$handle_type>,
            value: $value_type,
        }

        impl $name {
            pub fn new(handle: Handle<$handle_type>, value: $value_type) -> Self {
                Self { handle, value }
            }

            fn swap(&mut $self, $physics: &mut Physics) {
                 $apply_method
            }
        }

        impl Command for $name {


            fn name(&mut self, _context: &SceneContext) -> String {
                $human_readable_name.to_owned()
            }

            fn execute(&mut self, context: &mut SceneContext) {
                self.swap(&mut context.editor_scene.physics);
            }

            fn revert(&mut self, context: &mut SceneContext) {
                self.swap(&mut context.editor_scene.physics);
            }
        }
    };
}

macro_rules! define_body_command {
    ($name:ident($human_readable_name:expr, $value_type:ty) where fn swap($self:ident, $physics: ident, $body:ident) $apply_method:block ) => {
        define_physics_command!($name($human_readable_name, RigidBody, $value_type) where fn swap($self, $physics) {
            let $body = &mut $physics.bodies[$self.handle];
            $apply_method
        });
    };
}

macro_rules! define_collider_command {
    ($name:ident($human_readable_name:expr, $value_type:ty) where fn swap($self:ident, $physics:ident, $collider:ident) $apply_method:block ) => {
        define_physics_command!($name($human_readable_name, Collider, $value_type) where fn swap($self, $physics) {
            let $collider = &mut $physics.colliders[$self.handle];
            $apply_method
        });
    };
}

macro_rules! define_joint_command {
    ($name:ident($human_readable_name:expr, $value_type:ty) where fn swap($self:ident, $physics:ident, $joint:ident) $apply_method:block ) => {
        define_physics_command!($name($human_readable_name, Joint, $value_type) where fn swap($self, $physics) {
            let $joint = &mut $physics.joints[$self.handle];
            $apply_method
        });
    };
}

macro_rules! define_joint_variant_command {
    ($name:ident($human_readable_name:expr, $value_type:ty) where fn swap($self:ident, $physics:ident, $variant:ident, $var:ident) $apply_method:block ) => {
        define_physics_command!($name($human_readable_name, Joint, $value_type) where fn swap($self, $physics) {
            let joint = &mut $physics.joints[$self.handle];
            if let JointParamsDesc::$variant($var) = &mut joint.params {
                $apply_method
            } else {
                unreachable!();
            }
        });
    };
}

macro_rules! define_collider_variant_command {
    ($name:ident($human_readable_name:expr, $value_type:ty) where fn swap($self:ident, $physics:ident, $variant:ident, $var:ident) $apply_method:block ) => {
        define_physics_command!($name($human_readable_name, Collider, $value_type) where fn swap($self, $physics) {
            let collider = &mut $physics.colliders[$self.handle];
            if let ColliderShapeDesc::$variant($var) = &mut collider.shape {
                $apply_method
            } else {
                unreachable!();
            }
        });
    };
}

define_body_command!(SetBodyMassCommand("Set Body Mass", f32) where fn swap(self, physics, body) {
    std::mem::swap(&mut body.mass, &mut self.value);
});

define_collider_command!(SetColliderFrictionCommand("Set Collider Friction", f32) where fn swap(self, physics, collider) {
    std::mem::swap(&mut collider.friction, &mut self.value);
});

define_collider_command!(SetColliderRestitutionCommand("Set Collider Restitution", f32) where fn swap(self, physics, collider) {
    std::mem::swap(&mut collider.restitution, &mut self.value);
});

define_collider_command!(SetColliderPositionCommand("Set Collider Position", Vector3<f32>) where fn swap(self, physics, collider) {
    std::mem::swap(&mut collider.translation, &mut self.value);
});

define_collider_command!(SetColliderRotationCommand("Set Collider Rotation", UnitQuaternion<f32>) where fn swap(self, physics, collider) {
    std::mem::swap(&mut collider.rotation, &mut self.value);
});

define_collider_command!(SetColliderIsSensorCommand("Set Collider Is Sensor", bool) where fn swap(self, physics, collider) {
    std::mem::swap(&mut collider.is_sensor, &mut self.value);
});

define_collider_command!(SetColliderCollisionGroupsMembershipsCommand("Set Collider Collision Groups Memberships", u32) where fn swap(self, physics, collider) {
    std::mem::swap(&mut collider.collision_groups.memberships, &mut self.value);
});

define_collider_command!(SetColliderCollisionGroupsFilterCommand("Set Collider Collision Groups Filter", u32) where fn swap(self, physics, collider) {
    std::mem::swap(&mut collider.collision_groups.filter, &mut self.value);
});

define_collider_variant_command!(SetCylinderHalfHeightCommand("Set Cylinder Half Height", f32) where fn swap(self, physics, Cylinder, cylinder) {
    std::mem::swap(&mut cylinder.half_height, &mut self.value);
});

define_collider_variant_command!(SetCylinderRadiusCommand("Set Cylinder Radius", f32) where fn swap(self, physics, Cylinder, cylinder) {
    std::mem::swap(&mut cylinder.radius, &mut self.value);
});

define_collider_variant_command!(SetConeHalfHeightCommand("Set Cone Half Height", f32) where fn swap(self, physics, Cone, cone) {
    std::mem::swap(&mut cone.half_height, &mut self.value);
});

define_collider_variant_command!(SetConeRadiusCommand("Set Cone Radius", f32) where fn swap(self, physics, Cone, cone) {
    std::mem::swap(&mut cone.radius, &mut self.value);
});

define_collider_variant_command!(SetCuboidHalfExtentsCommand("Set Cuboid Half Extents", Vector3<f32>) where fn swap(self, physics, Cuboid, cuboid) {
    std::mem::swap(&mut cuboid.half_extents, &mut self.value);
});

define_collider_variant_command!(SetCapsuleRadiusCommand("Set Capsule Radius", f32) where fn swap(self, physics, Capsule, capsule) {
    std::mem::swap(&mut capsule.radius, &mut self.value);
});

define_collider_variant_command!(SetCapsuleBeginCommand("Set Capsule Begin", Vector3<f32>) where fn swap(self, physics, Capsule, capsule) {
    std::mem::swap(&mut capsule.begin, &mut self.value);
});

define_collider_variant_command!(SetCapsuleEndCommand("Set Capsule End", Vector3<f32>) where fn swap(self, physics, Capsule, capsule) {
    std::mem::swap(&mut capsule.end, &mut self.value);
});

define_collider_variant_command!(SetBallRadiusCommand("Set Ball Radius", f32) where fn swap(self, physics, Ball, ball) {
    std::mem::swap(&mut ball.radius, &mut self.value);
});

define_joint_variant_command!(SetBallJointAnchor1Command("Set Ball Joint Anchor 1", Vector3<f32>) where fn swap(self, physics, BallJoint, ball) {
    std::mem::swap(&mut ball.local_anchor1, &mut self.value);
});

define_joint_variant_command!(SetBallJointAnchor2Command("Set Ball Joint Anchor 2", Vector3<f32>) where fn swap(self, physics, BallJoint, ball) {
    std::mem::swap(&mut ball.local_anchor2, &mut self.value);
});

define_joint_variant_command!(SetFixedJointAnchor1TranslationCommand("Set Fixed Joint Anchor 1 Translation", Vector3<f32>) where fn swap(self, physics, FixedJoint, fixed) {
    std::mem::swap(&mut fixed.local_anchor1_translation, &mut self.value);
});

define_joint_variant_command!(SetFixedJointAnchor2TranslationCommand("Set Fixed Joint Anchor 2 Translation", Vector3<f32>) where fn swap(self, physics, FixedJoint, fixed) {
    std::mem::swap(&mut fixed.local_anchor2_translation, &mut self.value);
});

define_joint_variant_command!(SetFixedJointAnchor1RotationCommand("Set Fixed Joint Anchor 1 Rotation", UnitQuaternion<f32>) where fn swap(self, physics, FixedJoint, fixed) {
    std::mem::swap(&mut fixed.local_anchor1_rotation, &mut self.value);
});

define_joint_variant_command!(SetFixedJointAnchor2RotationCommand("Set Fixed Joint Anchor 2 Rotation", UnitQuaternion<f32>) where fn swap(self, physics, FixedJoint, fixed) {
    std::mem::swap(&mut fixed.local_anchor2_rotation, &mut self.value);
});

define_joint_variant_command!(SetRevoluteJointAnchor1Command("Set Revolute Joint Anchor 1", Vector3<f32>) where fn swap(self, physics, RevoluteJoint, revolute) {
    std::mem::swap(&mut revolute.local_anchor1, &mut self.value);
});

define_joint_variant_command!(SetRevoluteJointAxis1Command("Set Revolute Joint Axis 1", Vector3<f32>) where fn swap(self, physics, RevoluteJoint, revolute) {
    std::mem::swap(&mut revolute.local_axis1, &mut self.value);
});

define_joint_variant_command!(SetRevoluteJointAnchor2Command("Set Revolute Joint Anchor 2", Vector3<f32>) where fn swap(self, physics, RevoluteJoint, revolute) {
    std::mem::swap(&mut revolute.local_anchor2, &mut self.value);
});

define_joint_variant_command!(SetRevoluteJointAxis2Command("Set Prismatic Joint Axis 2", Vector3<f32>) where fn swap(self, physics, RevoluteJoint, revolute) {
    std::mem::swap(&mut revolute.local_axis2, &mut self.value);
});

define_joint_variant_command!(SetPrismaticJointAnchor1Command("Set Prismatic Joint Anchor 1", Vector3<f32>) where fn swap(self, physics, PrismaticJoint, prismatic) {
    std::mem::swap(&mut prismatic.local_anchor1, &mut self.value);
});

define_joint_variant_command!(SetPrismaticJointAxis1Command("Set Prismatic Joint Axis 1", Vector3<f32>) where fn swap(self, physics, PrismaticJoint, prismatic) {
    std::mem::swap(&mut prismatic.local_axis1, &mut self.value);
});

define_joint_variant_command!(SetPrismaticJointAnchor2Command("Set Prismatic Joint Anchor 2", Vector3<f32>) where fn swap(self, physics, PrismaticJoint, prismatic) {
    std::mem::swap(&mut prismatic.local_anchor2, &mut self.value);
});

define_joint_variant_command!(SetPrismaticJointAxis2Command("Set Prismatic Joint Axis 2", Vector3<f32>) where fn swap(self, physics, PrismaticJoint, prismatic) {
    std::mem::swap(&mut prismatic.local_axis2, &mut self.value);
});

define_joint_command!(SetJointConnectedBodyCommand("Set Joint Connected Body", ErasedHandle) where fn swap(self, physics, joint) {
    std::mem::swap(&mut joint.body2, &mut self.value);
});
