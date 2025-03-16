use bevy::prelude::*;

pub struct CharacterProfile {
    pub name: &'static str,
    pub description: &'static str,
    pub color: Color,
    pub health: u32,
    pub agility: f32,
    pub power: f32,
    pub defense: f32,
}

pub const CHARACTER_PROFILES: [CharacterProfile; 3] = [
    CharacterProfile {
        name: "Watson",
        description: "このゲームの開発者で、俊足が自慢のファイター。その代わり力は弱めなようだ。",
        color: Color::srgb(0.0, 0.0, 1.0),
        health: 400,
        agility: 4.0,
        power: 100.0,
        defense: 40.0
    },
    CharacterProfile {
        name: "Assasin",
        description: "身軽なアサシン。体力こそ少ないものの、二段ジャンプができるため回避に長けている。",
        color: Color::srgb(0.0, 1.0, 0.0),
        health: 300,
        agility: 3.0,
        power: 150.0,
        defense: 30.0
    },
    CharacterProfile {
        name: "Wrestler",
        description: "体力の多いレスラー。スピードは遅いが強靭な肉体とパワーで全てを解決する。",
        color: Color::srgb(1.0, 0.0, 0.0),
        health: 500,
        agility: 2.0,
        power: 250.0,
        defense: 50.0
    },
];