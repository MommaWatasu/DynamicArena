use bevy::prelude::*;

pub struct CharacterProfile {
    pub name: &'static str,
    pub description: &'static str,
    pub skill_name: &'static str,
    pub skill_description: &'static str,
    pub color: Color,
    pub health: u32,
    pub agility: f32,
    pub dexterity: f32,
    pub power: f32,
    pub defense: f32,
}

// TODO: edit dexterity
pub const CHARACTER_PROFILES: [CharacterProfile; 3] = [
    CharacterProfile {
        name: "Momma",
        description: "このゲームの開発者で、俊足が自慢のファイター。体力は低いが、強力な攻撃を素早く繰り出すことができる。",
        skill_name: "幻影歩法",
        skill_description: "一瞬で的に近づき、確定でダメージを与える",
        color: Color::srgb(0.0, 0.0, 1.0),
        health: 850,
        #[cfg(not(target_arch = "wasm32"))]
        agility: 4.0,
        #[cfg(target_arch = "wasm32")]
        agility: 2.0,
        dexterity: 1.0,
        power: 150.0,
        defense: 80.0
    },
    CharacterProfile {
        name: "Miyaguchi",
        description: "縁日班アトラク部門長で、バランス型のファイター。標準的なステータスで扱いやすい",
        skill_name: "魂吸収",
        skill_description: "相手に一定ダメージを与えて、その分自分が回復する",
        color: Color::srgb(0.0, 1.0, 0.0),
        health: 1000,
        #[cfg(not(target_arch = "wasm32"))]
        agility: 3.0,
        #[cfg(target_arch = "wasm32")]
        agility: 1.5,
        dexterity: 1.0,
        power: 100.0,
        defense: 100.0
    },
    CharacterProfile {
        name: "Matsumoto",
        description: "ボットの作成者で、体力の多いファイター。スピードは遅いが強靭な肉体とパワーで全てを解決する。",
        skill_name: "限界突破",
        skill_description: "スキル発動後最初の攻撃が大幅に強化される",
        color: Color::srgb(1.0, 0.0, 0.0),
        health: 1200,
        #[cfg(not(target_arch = "wasm32"))]
        agility: 2.0,
        #[cfg(target_arch = "wasm32")]
        agility: 1.0,
        dexterity: 1.0,
        power: 130.0,
        defense: 150.0
    },
];