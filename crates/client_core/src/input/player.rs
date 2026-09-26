//! Abstracted player input (keyboard + gamepad) and character movement.
//!
//! Actions are declared once in [`PlayerAction`] and bound for both keyboard
//! and gamepad here; game logic only reads [`ActionState`]. This keeps the door
//! open for remote/networked input later (see `plans/Player_Physics_And_Controls.md`).

use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::actor::ActorRoot;
use crate::physics::PlayerBody;
use crate::GameState;

/// Horizontal movement speed, world units per second.
const MOVE_SPEED: f32 = 4.0;
/// Jump impulse; tuned for roughly one tile (~1.0 unit) of height.
const JUMP_SPEED: f32 = 4.5;
/// Extra distance below the body used by the ground probe.
const GROUND_PROBE: f32 = 0.2;

/// Player actions, decoupled from any specific device.
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    /// Camera/world-relative movement (WASD, arrows, or left stick).
    #[actionlike(DualAxis)]
    Move,
    /// Jump (Space or gamepad South/A).
    Jump,
}

/// Registers the input manager plugin and the character controller systems.
pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_systems(
                Update,
                (attach_player_input, player_movement_system)
                    .chain()
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

fn default_input_map() -> InputMap<PlayerAction> {
    InputMap::default()
        .with_dual_axis(PlayerAction::Move, VirtualDPad::wasd())
        .with_dual_axis(PlayerAction::Move, VirtualDPad::arrow_keys())
        .with_dual_axis(PlayerAction::Move, GamepadStick::LEFT)
        .with(PlayerAction::Jump, KeyCode::Space)
        .with(PlayerAction::Jump, GamepadButton::South)
}

/// Gives every player body an input map and action state.
fn attach_player_input(
    mut commands: Commands,
    roots: Query<Entity, (With<ActorRoot>, Without<InputMap<PlayerAction>>)>,
) {
    for entity in roots.iter() {
        commands
            .entity(entity)
            .insert(default_input_map())
            .insert(ActionState::<PlayerAction>::default());
    }
}

/// Applies the input intent to the player's linear velocity.
///
/// Gravity and collisions are handled by Avian; this only controls the
/// horizontal velocity and the jump impulse when the ground probe hits.
fn player_movement_system(
    spatial: SpatialQuery,
    mut players: Query<
        (
            Entity,
            &ActionState<PlayerAction>,
            &PlayerBody,
            &Transform,
            &mut LinearVelocity,
        ),
        With<ActorRoot>,
    >,
) {
    for (entity, action, body, transform, mut velocity) in players.iter_mut() {
        let axis = action.clamped_axis_pair(&PlayerAction::Move);
        let mut direction = Vec3::new(axis.x, 0.0, -axis.y);
        if direction.length_squared() > 1.0 {
            direction = direction.normalize();
        }

        velocity.x = direction.x * MOVE_SPEED;
        velocity.z = direction.z * MOVE_SPEED;

        let filter = SpatialQueryFilter::from_excluded_entities([entity]);
        let probe = body.height * 0.5 + GROUND_PROBE;
        let grounded = spatial
            .cast_ray(
                transform.translation,
                Dir3::NEG_Y,
                probe,
                true,
                &filter,
            )
            .is_some();

        if grounded && action.just_pressed(&PlayerAction::Jump) {
            velocity.y = JUMP_SPEED;
        }
    }
}
