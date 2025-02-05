use std::ops::{Add, AddAssign, Div, Sub};

#[derive(Debug, Clone, Copy, Default)]
pub struct Pose {
    // true means right facing, false means left facing
    pub facing: bool,
    pub head: f32,
    pub body: f32,
    pub right_upper_arm: f32,
    pub right_lower_arm: f32,
    pub right_upper_leg: f32,
    pub right_lower_leg: f32,
    pub left_upper_arm: f32,
    pub left_lower_arm: f32,
    pub left_upper_leg: f32,
    pub left_lower_leg: f32,
}

impl Add for Pose {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            facing: self.facing,
            head: self.head + rhs.head,
            body: self.body + rhs.body,
            right_upper_arm: self.right_upper_arm + rhs.right_upper_arm,
            right_lower_arm: self.right_lower_arm + rhs.right_lower_arm,
            right_upper_leg: self.right_upper_leg + rhs.right_upper_leg,
            right_lower_leg: self.right_lower_leg + rhs.right_lower_leg,
            left_upper_arm: self.left_upper_arm + rhs.left_upper_arm,
            left_lower_arm: self.left_lower_arm + rhs.left_lower_arm,
            left_upper_leg: self.left_upper_leg + rhs.left_upper_leg,
            left_lower_leg: self.left_lower_leg + rhs.left_lower_leg,
        }
    }
}

impl AddAssign for Pose {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Pose {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            facing: self.facing,
            head: self.head - rhs.head,
            body: self.body - rhs.body,
            right_upper_arm: self.right_upper_arm - rhs.right_upper_arm,
            right_lower_arm: self.right_lower_arm - rhs.right_lower_arm,
            right_upper_leg: self.right_upper_leg - rhs.right_upper_leg,
            right_lower_leg: self.right_lower_leg - rhs.right_lower_leg,
            left_upper_arm: self.left_upper_arm - rhs.left_upper_arm,
            left_lower_arm: self.left_lower_arm - rhs.left_lower_arm,
            left_upper_leg: self.left_upper_leg - rhs.left_upper_leg,
            left_lower_leg: self.left_lower_leg - rhs.left_lower_leg,
        }
    }
}

impl Div<f32> for Pose {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            facing: self.facing,
            head: self.head / rhs,
            body: self.body / rhs,
            right_upper_arm: self.right_upper_arm / rhs,
            right_lower_arm: self.right_lower_arm / rhs,
            right_upper_leg: self.right_upper_leg / rhs,
            right_lower_leg: self.right_lower_leg / rhs,
            left_upper_arm: self.left_upper_arm / rhs,
            left_lower_arm: self.left_lower_arm / rhs,
            left_upper_leg: self.left_upper_leg / rhs,
            left_lower_leg: self.left_lower_leg / rhs,
        }
    }
}

pub const OPPOSITE_DEFAULT_POSE: Pose = Pose {
    facing: false,
    head: 0.0,
    body: 0.0,
    right_upper_arm: 10.0,
    right_lower_arm: 90.0,
    right_upper_leg: 10.0,
    right_lower_leg: -40.0,
    left_upper_arm: 30.0,
    left_lower_arm: 90.0,
    left_upper_leg: 40.0,
    left_lower_leg: -50.0,
};

pub const IDLE_POSE1: Pose = Pose {
    facing: true,
    head: 0.0,
    body: 0.0,
    right_upper_arm: 10.0,
    right_lower_arm: 90.0,
    right_upper_leg: 10.0,
    right_lower_leg: -40.0,
    left_upper_arm: 30.0,
    left_lower_arm: 90.0,
    left_upper_leg: 40.0,
    left_lower_leg: -50.0,
};

pub const IDLE_POSE2: Pose = Pose {
    facing: true,
    head: 0.0,
    body: 0.0,
    right_upper_arm: 10.0,
    right_lower_arm: 90.0,
    right_upper_leg: 20.0,
    right_lower_leg: -50.0,
    left_upper_arm: 30.0,
    left_lower_arm: 90.0,
    left_upper_leg: 50.0,
    left_lower_leg: -60.0,
};

pub const RUNNING_POSE1: Pose = Pose {
    facing: true,
    head: 0.0,
    body: -10.0,
    right_upper_arm: -60.0,
    right_lower_arm: 90.0,
    right_upper_leg: 100.0,
    right_lower_leg: -90.0,
    left_upper_arm: 60.0,
    left_lower_arm: 90.0,
    left_upper_leg: -30.0,
    left_lower_leg: -50.0,
};

pub const RUNNING_POSE2: Pose = Pose {
    facing: true,
    head: 0.0,
    body: -10.0,
    right_upper_arm: 60.0,
    right_lower_arm: 90.0,
    right_upper_leg: -30.0,
    right_lower_leg: -50.0,
    left_upper_arm: -60.0,
    left_lower_arm: 90.0,
    left_upper_leg: 100.0,
    left_lower_leg: -90.0,
};

pub const JUMPING_POSE1: Pose = Pose {
    facing: true,
    head: 0.0,
    body: 0.0,
    right_upper_arm: 20.0,
    right_lower_arm: -20.0,
    right_upper_leg: 10.0,
    right_lower_leg: -10.0,
    left_upper_arm: -20.0,
    left_lower_arm: 10.0,
    left_upper_leg: 60.0,
    left_lower_leg: -70.0,
};

pub const JUMPING_POSE2: Pose = Pose {
    facing: true,
    head: 0.0,
    body: 0.0,
    right_upper_arm: 40.0,
    right_lower_arm: -20.0,
    right_upper_leg: 10.0,
    right_lower_leg: -10.0,
    left_upper_arm: -40.0,
    left_lower_arm: 20.0,
    left_upper_leg: 60.0,
    left_lower_leg: -70.0,
};

pub const JUMPING_KICK_POSE: Pose = Pose {
    facing: true,
    head: 0.0,
    body: -10.0,
    right_upper_arm: -10.0,
    right_lower_arm: 20.0,
    right_upper_leg: 70.0,
    right_lower_leg: -110.0,
    left_upper_arm: -20.0,
    left_lower_arm: 10.0,
    left_upper_leg: 10.0,
    left_lower_leg: -10.0,
};

pub const KICK_POSE: Pose = Pose {
    facing: true,
    head: 0.0,
    body: 10.0,
    right_upper_arm: 10.0,
    right_lower_arm: 90.0,
    right_upper_leg: 100.0,
    right_lower_leg: 10.0,
    left_upper_arm: 30.0,
    left_lower_arm: 90.0,
    left_upper_leg: -20.0,
    left_lower_leg: -10.0,
};

pub const HIGH_KICK_POSE: Pose = Pose {
    facing: true,
    head: 0.0,
    body: 10.0,
    right_upper_arm: 10.0,
    right_lower_arm: 90.0,
    right_upper_leg: 150.0,
    right_lower_leg: 10.0,
    left_upper_arm: 30.0,
    left_lower_arm: 90.0,
    left_upper_leg: -20.0,
    left_lower_leg: -10.0,
};

pub const PUNCH_POSE: Pose = Pose {
    facing: true,
    head: 0.0,
    body: 0.0,
    right_upper_arm: 90.0,
    right_lower_arm: 0.0,
    right_upper_leg: 10.0,
    right_lower_leg: -40.0,
    left_upper_arm: 30.0,
    left_lower_arm: 90.0,
    left_upper_leg: 40.0,
    left_lower_leg: -50.0,
};

pub const UPPER_PUNCH_POSE1: Pose = Pose {
    facing: true,
    head: 0.0,
    body: 0.0,
    right_upper_arm: 90.0,
    right_lower_arm: 0.0,
    right_upper_leg: 10.0,
    right_lower_leg: -40.0,
    left_upper_arm: 30.0,
    left_lower_arm: 90.0,
    left_upper_leg: 40.0,
    left_lower_leg: -50.0,
};

pub const UPPER_PUNCH_POSE2: Pose = Pose {
    facing: true,
    head: 0.0,
    body: 0.0,
    right_upper_arm: 90.0,
    right_lower_arm: 90.0,
    right_upper_leg: 10.0,
    right_lower_leg: -40.0,
    left_upper_arm: 30.0,
    left_lower_arm: 90.0,
    left_upper_leg: 40.0,
    left_lower_leg: -50.0,
};