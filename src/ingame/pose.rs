use std::ops::{Add, AddAssign, Div, Sub};

// number of frames for each action
pub const FRAMES_IDLE: usize = 57;
pub const FRAMES_WALK: usize = 30;
pub const FRAMES_JUMP: usize = 58;
pub const FRAMES_KICK: usize = 46;
pub const FRAMES_PUNCH: usize = 32;
pub const FRAMES_BACK_KICK: usize = 49;
pub const FRAMES_BEND_DOWN: usize = 50;
pub const FRAMES_ROLL: usize = 36;
pub const FRAMES_VICTORY: usize = 93;
pub const FRAMES_DEFEATED: usize = 120;
pub const FRAMES_ATTACKED: usize = 25;

// The pose of a character
// facing: true means right facing, false means left facing
// offset: the offset of the character from the center of the body([x, y])
// old_offset: the offset of the character from the center of the body in the previous frame
#[derive(Debug, Clone, Copy, Default)]
pub struct Pose {
    // true means right facing, false means left facing
    pub facing: bool,
    pub offset: [f32; 2],
    pub old_offset: [f32; 2],
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

impl Pose {
    pub fn set(&mut self, other: Pose) {
        self.offset = other.offset;
        self.old_offset = other.old_offset;
        self.head = other.head;
        self.body = other.body;
        self.right_upper_arm = other.right_upper_arm;
        self.right_lower_arm = other.right_lower_arm;
        self.right_upper_leg = other.right_upper_leg;
        self.right_lower_leg = other.right_lower_leg;
        self.left_upper_arm = other.left_upper_arm;
        self.left_lower_arm = other.left_lower_arm;
        self.left_upper_leg = other.left_upper_leg;
        self.left_lower_leg = other.left_lower_leg;
    }
}

impl Add for Pose {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            facing: self.facing,
            offset: [
                self.offset[0] + rhs.offset[0],
                self.offset[1] + rhs.offset[1],
            ],
            old_offset: self.old_offset,
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
            offset: [
                self.offset[0] - rhs.offset[0],
                self.offset[1] - rhs.offset[1],
            ],
            old_offset: self.old_offset,
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
            offset: [self.offset[0] / rhs, self.offset[1] / rhs],
            old_offset: self.old_offset,
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
    offset: [0.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 30.0,
    body: -5.0,
    right_upper_arm: 0.0,
    right_lower_arm: 120.0,
    right_upper_leg: -40.0,
    right_lower_leg: 40.0,
    left_upper_arm: 30.0,
    left_lower_arm: 90.0,
    left_upper_leg: 60.0,
    left_lower_leg: -50.0,
};

pub const IDLE_POSE1: Pose = Pose {
    facing: true,
    offset: [10.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 40.0,
    body: -5.0,
    right_upper_arm: 0.0,
    right_lower_arm: 120.0,
    right_upper_leg: -10.0,
    right_lower_leg: -30.0,
    left_upper_arm: 20.0,
    left_lower_arm: 90.0,
    left_upper_leg: 60.0,
    left_lower_leg: -50.0,
};

pub const IDLE_POSE2: Pose = Pose {
    facing: true,
    offset: [10.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 40.0,
    body: -5.0,
    right_upper_arm: -10.0,
    right_lower_arm: 110.0,
    right_upper_leg: -10.0,
    right_lower_leg: -30.0,
    left_upper_arm: 20.0,
    left_lower_arm: 80.0,
    left_upper_leg: 60.0,
    left_lower_leg: -50.0,
};

pub const WALKING_POSE1: Pose = Pose {
    facing: true,
    offset: [10.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 20.0,
    body: 0.0,
    right_upper_arm: 20.0,
    right_lower_arm: 150.0,
    right_upper_leg: 15.0,
    right_lower_leg: -30.0,
    left_upper_arm: 30.0,
    left_lower_arm: 120.0,
    left_upper_leg: 20.0,
    left_lower_leg: -40.0,
};

pub const WALKING_POSE2: Pose = Pose {
    facing: true,
    offset: [10.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 20.0,
    body: 0.0,
    right_upper_arm: 20.0,
    right_lower_arm: 150.0,
    right_upper_leg: -10.0,
    right_lower_leg: -20.0,
    left_upper_arm: 30.0,
    left_lower_arm: 120.0,
    left_upper_leg: 50.0,
    left_lower_leg: -50.0,
};

pub const BEND_DOWN_POSE1: Pose = Pose {
    facing: true,
    offset: [10.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 0.0,
    body: 0.0,
    right_upper_arm: 0.0,
    right_lower_arm: 0.0,
    right_upper_leg: 0.0,
    right_lower_leg: 0.0,
    left_upper_arm: 0.0,
    left_lower_arm: 0.0,
    left_upper_leg: 0.0,
    left_lower_leg: 0.0,
};

pub const BEND_DOWN_POSE2: Pose = Pose {
    facing: true,
    offset: [0.0, -60.0],
    old_offset: [0.0, 0.0],
    head: 20.0,
    body: -10.0,
    right_upper_arm: 10.0,
    right_lower_arm: 100.0,
    right_upper_leg: 100.0,
    right_lower_leg: -120.0,
    left_upper_arm: 10.0,
    left_lower_arm: 100.0,
    left_upper_leg: 100.0,
    left_lower_leg: -120.0,
};

pub const ROLL_FORWARD_POSE1: Pose = Pose {
    facing: true,
    offset: [30.0, -10.0],
    old_offset: [0.0, 0.0],
    head: -10.0,
    body: -30.0,
    right_upper_arm: -50.0,
    right_lower_arm: 120.0,
    right_upper_leg: 20.0,
    right_lower_leg: -50.0,
    left_upper_arm: 30.0,
    left_lower_arm: 120.0,
    left_upper_leg: 60.0,
    left_lower_leg: -110.0,
};

pub const ROLL_FORWARD_POSE2: Pose = Pose {
    facing: true,
    offset: [100.0, -140.0],
    old_offset: [0.0, 0.0],
    head: 20.0,
    body: -90.0,
    right_upper_arm: 120.0,
    right_lower_arm: 40.0,
    right_upper_leg: 85.0,
    right_lower_leg: -100.0,
    left_upper_arm: 30.0,
    left_lower_arm: 100.0,
    left_upper_leg: 50.0,
    left_lower_leg: -30.0,
};

pub const ROLL_FORWARD_POSE3: Pose = Pose {
    facing: true,
    offset: [50.0, -250.0],
    old_offset: [0.0, 0.0],
    head: 100.0,
    body: -190.0,
    right_upper_arm: 110.0,
    right_lower_arm: -110.0,
    right_upper_leg: 90.0,
    right_lower_leg: -110.0,
    left_upper_arm: 100.0,
    left_lower_arm: -120.0,
    left_upper_leg: 80.0,
    left_lower_leg: -120.0,
};

pub const ROLL_FORWARD_POSE4: Pose = Pose {
    facing: true,
    offset: [-10.0, -130.0],
    old_offset: [0.0, 0.0],
    head: 30.0,
    body: -330.0,
    right_upper_arm: -40.0,
    right_lower_arm: 90.0,
    right_upper_leg: 70.0,
    right_lower_leg: -40.0,
    left_upper_arm: 30.0,
    left_lower_arm: 60.0,
    left_upper_leg: 100.0,
    left_lower_leg: -100.0,
};

pub const ROLL_FORWARD_POSE5: Pose = Pose {
    facing: true,
    offset: [0.0, -50.0],
    old_offset: [0.0, 0.0],
    head: 30.0,
    body: -30.0,
    right_upper_arm: -50.0,
    right_lower_arm: 90.0,
    right_upper_leg: 30.0,
    right_lower_leg: -110.0,
    left_upper_arm: -10.0,
    left_lower_arm: 80.0,
    left_upper_leg: 120.0,
    left_lower_leg: -110.0,
};

pub const ROLL_FORWARD_POSE6: Pose = Pose {
    facing: true,
    offset: [0.0, -10.0],
    old_offset: [0.0, 0.0],
    head: -10.0,
    body: -20.0,
    right_upper_arm: -70.0,
    right_lower_arm: 130.0,
    right_upper_leg: 30.0,
    right_lower_leg: -50.0,
    left_upper_arm: 30.0,
    left_lower_arm: 140.0,
    left_upper_leg: 90.0,
    left_lower_leg: -110.0,
};

// roll back movement is just playing the forward movement in reverse
pub const ROLL_BACK_POSE1: Pose = ROLL_FORWARD_POSE6;
pub const ROLL_BACK_POSE2: Pose = ROLL_FORWARD_POSE5;
pub const ROLL_BACK_POSE3: Pose = ROLL_FORWARD_POSE4;
pub const ROLL_BACK_POSE4: Pose = ROLL_FORWARD_POSE3;
pub const ROLL_BACK_POSE5: Pose = ROLL_FORWARD_POSE2;
pub const ROLL_BACK_POSE6: Pose = ROLL_FORWARD_POSE1;

pub const JUMP_POSE1: Pose = Pose {
    facing: true,
    offset: [0.0, 50.0],
    old_offset: [0.0, 0.0],
    head: 20.0,
    body: 0.0,
    right_upper_arm: -10.0,
    right_lower_arm: 20.0,
    right_upper_leg: 0.0,
    right_lower_leg: -20.0,
    left_upper_arm: -5.0,
    left_lower_arm: 45.0,
    left_upper_leg: 20.0,
    left_lower_leg: -30.0,
};

pub const JUMP_POSE2: Pose = Pose {
    facing: true,
    offset: [20.0, -60.0],
    old_offset: [0.0, 0.0],
    head: 20.0,
    body: -20.0,
    right_upper_arm: -30.0,
    right_lower_arm: 40.0,
    right_upper_leg: 60.0,
    right_lower_leg: -100.0,
    left_upper_arm: 0.0,
    left_lower_arm: 50.0,
    left_upper_leg: 120.0,
    left_lower_leg: -120.0,
};

pub const JUMP_POSE3: Pose = Pose {
    facing: true,
    offset: [20.0, 100.0],
    old_offset: [0.0, 0.0],
    head: 20.0,
    body: -30.0,
    right_upper_arm: 270.0,
    right_lower_arm: -20.0,
    right_upper_leg: 100.0,
    right_lower_leg: -120.0,
    left_upper_arm: 80.0,
    left_lower_arm: 30.0,
    left_upper_leg: 90.0,
    left_lower_leg: -120.0,
};

pub const JUMP_KICK_POSE: Pose = Pose {
    facing: true,
    offset: [-40.0, 30.0],
    old_offset: [0.0, 0.0],
    head: 40.0,
    body: -10.0,
    right_upper_arm: -90.0,
    right_lower_arm: 150.0,
    right_upper_leg: -60.0,
    right_lower_leg: 130.0,
    left_upper_arm: 80.0,
    left_lower_arm: 0.0,
    left_upper_leg: 80.0,
    left_lower_leg: 0.0,
};

pub const KICK_POSE1: Pose = Pose {
    facing: true,
    offset: [0.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 0.0,
    body: 0.0,
    right_upper_arm: 100.0,
    right_lower_arm: 150.0,
    right_upper_leg: 100.0,
    right_lower_leg: -150.0,
    left_upper_arm: -10.0,
    left_lower_arm: 90.0,
    left_upper_leg: 0.0,
    left_lower_leg: -10.0,
};

pub const KICK_POSE2: Pose = Pose {
    facing: true,
    offset: [0.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 20.0,
    body: 0.0,
    right_upper_arm: -80.0,
    right_lower_arm: 50.0,
    right_upper_leg: 85.0,
    right_lower_leg: 0.0,
    left_upper_arm: 80.0,
    left_lower_arm: 90.0,
    left_upper_leg: 10.0,
    left_lower_leg: -10.0,
};

pub const BACK_KICK_POSE1: Pose = Pose {
    facing: true,
    offset: [10.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 40.0,
    body: -5.0,
    right_upper_arm: 0.0,
    right_lower_arm: 120.0,
    right_upper_leg: -10.0,
    right_lower_leg: -30.0,
    left_upper_arm: 20.0,
    left_lower_arm: 90.0,
    left_upper_leg: 60.0,
    left_lower_leg: -50.0,
};

pub const BACK_KICK_POSE2: Pose = Pose {
    facing: true,
    offset: [10.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 30.0,
    body: 10.0,
    right_upper_arm: -20.0,
    right_lower_arm: 130.0,
    right_upper_leg: -20.0,
    right_lower_leg: 10.0,
    left_upper_arm: 60.0,
    left_lower_arm: -100.0,
    left_upper_leg: 10.0,
    left_lower_leg: -30.0,
};

pub const BACK_KICK_POSE3: Pose = Pose {
    facing: true,
    offset: [10.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 40.0,
    body: 40.0,
    right_upper_arm: -60.0,
    right_lower_arm: 110.0,
    right_upper_leg: -50.0,
    right_lower_leg: 20.0,
    left_upper_arm: 60.0,
    left_lower_arm: -30.0,
    left_upper_leg: 60.0,
    left_lower_leg: 0.0,
};

pub const PUNCH_POSE: Pose = Pose {
    facing: true,
    offset: [0.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 0.0,
    body: 0.0,
    right_upper_arm: -40.0,
    right_lower_arm: 120.0,
    right_upper_leg: -40.0,
    right_lower_leg: 40.0,
    left_upper_arm: 90.0,
    left_lower_arm: 0.0,
    left_upper_leg: 60.0,
    left_lower_leg: -60.0,
};

pub const THUNDER_PUNCH_POSE: Pose = Pose {
    facing: true,
    offset: [0.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 0.0,
    body: -40.0,
    right_upper_arm: -80.0,
    right_lower_arm: 160.0,
    right_upper_leg: -50.0,
    right_lower_leg: -20.0,
    left_upper_arm: 100.0,
    left_lower_arm: -30.0,
    left_upper_leg: 10.0,
    left_lower_leg: -60.0,
};

pub const HAMMER_POSE1: Pose = Pose {
    facing: true,
    offset: [0.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 0.0,
    body: 0.0,
    right_upper_arm: -10.0,
    right_lower_arm: 140.0,
    right_upper_leg: -40.0,
    right_lower_leg: 40.0,
    left_upper_arm: 120.0,
    left_lower_arm: 60.0,
    left_upper_leg: 60.0,
    left_lower_leg: -60.0,
};

pub const HAMMER_POSE2: Pose = Pose {
    facing: true,
    offset: [0.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 0.0,
    body: 0.0,
    right_upper_arm: -10.0,
    right_lower_arm: 140.0,
    right_upper_leg: -40.0,
    right_lower_leg: 40.0,
    left_upper_arm: 50.0,
    left_lower_arm: 0.0,
    left_upper_leg: 60.0,
    left_lower_leg: -60.0,
};

pub const STUN_POSE1: Pose = Pose {
    facing: true,
    offset: [10.0, 0.0],
    old_offset: [0.0, 0.0],
    head: 20.0,
    body: 0.0,
    right_upper_arm: 20.0,
    right_lower_arm: 150.0,
    right_upper_leg: 15.0,
    right_lower_leg: -30.0,
    left_upper_arm: 30.0,
    left_lower_arm: 120.0,
    left_upper_leg: 20.0,
    left_lower_leg: -40.0,
};

pub const STUN_POSE2: Pose = Pose {
    facing: true,
    offset: [10.0, 0.0],
    old_offset: [0.0, 0.0],
    head: -40.0,
    body: 15.0,
    right_upper_arm: -10.0,
    right_lower_arm: 130.0,
    right_upper_leg: -20.0,
    right_lower_leg: -40.0,
    left_upper_arm: 10.0,
    left_lower_arm: 90.0,
    left_upper_leg: 30.0,
    left_lower_leg: -50.0,
};