use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::time::Duration;

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct AnimationData {
    /// All motion paths in the animation
    // pub paths: Vec<SkeletonMotionPath>,
    pub id: String,
    /// id of the associated polygon
    pub polygon_id: String,
    /// Total duration of the animation
    pub duration: Duration,
    /// Hierarchical property structure for UI
    pub properties: Vec<AnimationProperty>,
}

/// Represents a property that can be animated in the UI
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct AnimationProperty {
    /// Name of the property (e.g., "Position.X", "Rotation.Z")
    pub name: String,
    /// Path to this property in the data (for linking to MotionPath data)
    pub property_path: String,
    /// Nested properties (if any)
    pub children: Vec<AnimationProperty>,
    /// Direct keyframes for this property
    pub keyframes: Vec<UIKeyframe>,
    /// Visual depth in the property tree
    pub depth: u32,
}

/// Types of easing functions available for interpolation
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

/// Represents a keyframe in the UI
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub struct UIKeyframe {
    /// Used to associate with this speciifc UI Keyframe
    pub id: String,
    /// Used to associate with the SkeletonKeyframe for updates
    // pub skel_key_id: String,
    /// Time of the keyframe
    pub time: Duration,
    /// Value at this keyframe (could be position, rotation, etc)
    pub value: KeyframeValue,
    /// Type of interpolation to next keyframe
    pub easing: EasingType,
}

/// Possible values for keyframes
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug)]
pub enum KeyframeValue {
    // Position([f32; 3]),
    // Rotation([f32; 4]),
    // Scale([f32; 3]),
    // Custom(Vec<f32>),
    Position([i32; 2]),
    Rotation(i32),
    Scale(i32), // this will be 100 for default size to work with i32 and Eq
    PerspectiveX(i32),
    PerspectiveY(i32),
    Opacity(i32), // also out of 100
    Custom(Vec<i32>),
}
