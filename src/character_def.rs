use bevy::prelude::*;

pub struct CharacterProfile {
    pub name: &'static str,
    pub description: &'static str,
    pub color: Color,
    pub health: u32,
    pub speed: f32,
    pub power: u32,
}

pub const CHARACTER_PROFILES: [CharacterProfile; 3] = [
    CharacterProfile {
        name: "Watson",
        description: "このゲームの開発者で、俊足が自慢のファイター。足が速い変わりに力は弱めなようだ。",
        color: Color::srgb(0.0, 0.0, 1.0),
        health: 400,
        speed: 8.0,
        power: 8,
    },
    CharacterProfile {
        name: "Assasin",
        description: "身軽なアサシン。体力こそ少ないものの、二段ジャンプができるため回避に長けている。",
        color: Color::srgb(0.0, 1.0, 0.0),
        health: 300,
        speed: 5.0,
        power: 10
    },
    CharacterProfile {
        name: "Wrestler",
        description: "体力の多いレスラー。スピードは遅いが強靭な肉体とパワーで全てを解決する。",
        color: Color::srgb(1.0, 0.0, 0.0),
        health: 500,
        speed: 3.0,
        power: 15
    },
];