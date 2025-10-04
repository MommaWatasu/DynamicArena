use bevy::prelude::*;

pub const ENERGY_MAX: u8 = 100;
pub const FIRE_CHARGE_MAX: u16 = 300;

// TODO: update radar chart(remove dexterity and add the power of skill)
pub struct CharacterProfile {
    pub name: &'static str,
    pub description: &'static str,
    pub skill_name: &'static str,
    pub skill_description: &'static str,
    pub health: u32,
    pub agility: f32,
    pub power: f32,
    pub defense: f32,
}

pub const SOUL_COLOR: [Color; 3] = [
    Color::srgb(0.0, 20.0, 18.0),
    Color::srgb(0.0, 20.0, 15.0),
    Color::srgb(1.0, 15.0, 20.0),
];

// FIXME: jumping motion of Momma is broken bacause of its dexterity
pub const CHARACTER_PROFILES: [CharacterProfile; 3] = [
    CharacterProfile {
        name: "Momma",
        description: "このゲームの開発者で、俊足が自慢のファイター。体力は低いが、強力な攻撃を素早く繰り出すことができる。",
        skill_name: "神速雷光",
        skill_description: "一瞬で敵に近づき、確定でダメージを与える",
        health: 850,
        #[cfg(not(feature="phone"))]
        agility: 4.0,
        #[cfg(feature="phone")]
        agility: 2.0,
        power: 150.0,
        defense: 80.0
    },
    CharacterProfile {
        name: "Miyaguchi",
        description: "縁日班アトラク部門長で、バランス型のファイター。標準的なステータスで扱いやすい",
        skill_name: "魂吸収",
        skill_description: "相手に一定ダメージを与えて、その分自分が回復する",
        health: 1000,
        #[cfg(not(feature="phone"))]
        agility: 3.0,
        #[cfg(feature="phone")]
        agility: 1.5,
        power: 100.0,
        defense: 100.0
    },
    CharacterProfile {
        name: "Matsumoto",
        description: "ボットの作成者で、体力の多いファイター。スピードは遅いが強靭な肉体とパワーで全てを解決する。",
        skill_name: "鉄拳制裁",
        skill_description: "巨大な拳で相手をたたき、地面にいる敵に大ダメージを与える",
        health: 1200,
        #[cfg(not(feature="phone"))]
        agility: 2.8,
        #[cfg(feature="phone")]
        agility: 1.0,
        power: 130.0,
        defense: 150.0
    },
];
