#![allow(dead_code)]
//! Игровая логика без зависимостей: юниты, способности, крипы, башни, ИИ врага.
//! Координаты: x вдоль линии (0..W), y поперёк (0..H), линия проходит по y = LY.

pub const W: f32 = 3200.0;
pub const H: f32 = 1000.0;
pub const LY: f32 = 500.0;
pub const TEAM_COL: [[f32; 3]; 2] = [[0.29, 0.64, 1.0], [1.0, 0.35, 0.35]];
pub const TEAM_NAME: [&str; 2] = ["Лазурный легион", "Багровый орден"];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Hero,
    Creep,
    Tower,
    Throne,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AbType {
    SelfAoe,
    Buff,
    Skillshot,
    AoePoint,
    Dash,
    Blink,
}

#[derive(Clone, Copy)]
pub struct AbDef {
    pub key: char,
    pub name: &'static str,
    pub ty: AbType,
    pub range: f32,
    pub speed: f32,
    pub width: f32,
    pub radius: f32,
    pub delay: f32,
    pub cd: f32,
    pub mana: f32,
    pub dmg: f32,
    pub slow: f32,
    pub slow_amt: f32,
    pub stun: f32,
    pub dur: f32,
    pub atk_speed: f32,
    pub mv: f32,
    pub pierce: bool,
}

const AB0: AbDef = AbDef {
    key: ' ',
    name: "",
    ty: AbType::SelfAoe,
    range: 0.0,
    speed: 0.0,
    width: 0.0,
    radius: 0.0,
    delay: 0.0,
    cd: 0.0,
    mana: 0.0,
    dmg: 0.0,
    slow: 0.0,
    slow_amt: 1.0,
    stun: 0.0,
    dur: 0.0,
    atk_speed: 1.0,
    mv: 0.0,
    pierce: false,
};

pub struct HeroDef {
    pub name: &'static str,
    pub glyph: &'static str,
    pub role: &'static str,
    pub color: [f32; 3],
    pub hp: f32,
    pub mp: f32,
    pub dmg: f32,
    pub range: f32,
    pub interval: f32,
    pub speed: f32,
    pub proj: f32,
    pub r: f32,
    pub abilities: [AbDef; 4],
}

pub const HEROES: [HeroDef; 3] = [
    HeroDef {
        name: "Борей",
        glyph: "Б",
        role: "Страж, ближний бой",
        color: [0.35, 0.66, 1.0],
        hp: 640.0,
        mp: 220.0,
        dmg: 46.0,
        range: 58.0,
        interval: 0.9,
        speed: 215.0,
        proj: 0.0,
        r: 22.0,
        abilities: [
            AbDef {
                key: 'Q',
                name: "Рывок",
                ty: AbType::Dash,
                range: 440.0,
                cd: 9.0,
                mana: 60.0,
                dmg: 70.0,
                radius: 120.0,
                stun: 0.9,
                ..AB0
            },
            AbDef {
                key: 'W',
                name: "Удар земли",
                ty: AbType::SelfAoe,
                cd: 11.0,
                mana: 70.0,
                dmg: 90.0,
                radius: 190.0,
                slow: 2.5,
                slow_amt: 0.5,
                ..AB0
            },
            AbDef {
                key: 'E',
                name: "Боевой клич",
                ty: AbType::Buff,
                cd: 16.0,
                mana: 50.0,
                dur: 5.0,
                atk_speed: 1.6,
                mv: 55.0,
                ..AB0
            },
            AbDef {
                key: 'R',
                name: "Гнев титана",
                ty: AbType::SelfAoe,
                cd: 45.0,
                mana: 140.0,
                dmg: 260.0,
                radius: 270.0,
                stun: 1.6,
                ..AB0
            },
        ],
    },
    HeroDef {
        name: "Лиора",
        glyph: "Л",
        role: "Чародейка, дальний бой",
        color: [0.71, 0.55, 1.0],
        hp: 440.0,
        mp: 380.0,
        dmg: 38.0,
        range: 430.0,
        interval: 1.0,
        speed: 200.0,
        proj: 650.0,
        r: 20.0,
        abilities: [
            AbDef {
                key: 'Q',
                name: "Ледяной шар",
                ty: AbType::Skillshot,
                range: 680.0,
                speed: 850.0,
                width: 26.0,
                cd: 6.0,
                mana: 60.0,
                dmg: 110.0,
                slow: 2.0,
                slow_amt: 0.55,
                ..AB0
            },
            AbDef {
                key: 'W',
                name: "Снежная буря",
                ty: AbType::AoePoint,
                range: 620.0,
                radius: 170.0,
                delay: 0.45,
                cd: 12.0,
                mana: 90.0,
                dmg: 120.0,
                slow: 3.0,
                slow_amt: 0.45,
                ..AB0
            },
            AbDef {
                key: 'E',
                name: "Мерцание",
                ty: AbType::Blink,
                range: 460.0,
                cd: 10.0,
                mana: 50.0,
                ..AB0
            },
            AbDef {
                key: 'R',
                name: "Метеор",
                ty: AbType::AoePoint,
                range: 720.0,
                radius: 240.0,
                delay: 1.0,
                cd: 50.0,
                mana: 180.0,
                dmg: 340.0,
                stun: 1.0,
                ..AB0
            },
        ],
    },
    HeroDef {
        name: "Тэйн",
        glyph: "Т",
        role: "Стрелок, дальний бой",
        color: [0.44, 0.89, 0.60],
        hp: 500.0,
        mp: 270.0,
        dmg: 52.0,
        range: 520.0,
        interval: 0.8,
        speed: 225.0,
        proj: 950.0,
        r: 20.0,
        abilities: [
            AbDef {
                key: 'Q',
                name: "Меткий выстрел",
                ty: AbType::Skillshot,
                range: 820.0,
                speed: 1250.0,
                width: 22.0,
                cd: 7.0,
                mana: 50.0,
                dmg: 125.0,
                ..AB0
            },
            AbDef {
                key: 'W',
                name: "Азарт",
                ty: AbType::Buff,
                cd: 14.0,
                mana: 50.0,
                dur: 5.0,
                atk_speed: 1.7,
                mv: 0.0,
                ..AB0
            },
            AbDef {
                key: 'E',
                name: "Прыжок",
                ty: AbType::Dash,
                range: 380.0,
                cd: 9.0,
                mana: 40.0,
                ..AB0
            },
            AbDef {
                key: 'R',
                name: "Пронзающая стрела",
                ty: AbType::Skillshot,
                range: 1250.0,
                speed: 1500.0,
                width: 36.0,
                cd: 40.0,
                mana: 150.0,
                dmg: 300.0,
                stun: 0.5,
                pierce: true,
                ..AB0
            },
        ],
    },
];

pub struct Diff {
    pub name: &'static str,
    pub dmg: f32,
    pub hp: f32,
    pub gold: f32,
    pub think: f32,
}
pub const DIFFS: [Diff; 3] = [
    Diff {
        name: "Новичок",
        dmg: 0.75,
        hp: 0.85,
        gold: 0.7,
        think: 0.6,
    },
    Diff {
        name: "Воин",
        dmg: 1.0,
        hp: 1.0,
        gold: 1.0,
        think: 1.0,
    },
    Diff {
        name: "Легенда",
        dmg: 1.25,
        hp: 1.25,
        gold: 1.35,
        think: 1.4,
    },
];

pub const ITEM_NAMES: [&str; 4] = [
    "Клинок мощи (+12 урона)",
    "Доспех жизни (+160 здоровья)",
    "Сапоги ветра (+22 скорости)",
    "Кристалл маны (+120 маны)",
];
const ITEM_BASE: [f32; 4] = [260.0, 300.0, 240.0, 200.0];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Order {
    None,
    Move(f32, f32),
    Attack(u32),
}

#[derive(Clone, Copy)]
pub struct Dash {
    pub sx: f32,
    pub sy: f32,
    pub tx: f32,
    pub ty: f32,
    pub t: f32,
    pub dur: f32,
    pub ab: usize,
    pub pw: f32,
}

#[derive(Clone)]
pub struct HeroData {
    pub key: usize,
    pub bot: bool,
    pub mp: f32,
    pub max_mp: f32,
    pub level: u32,
    pub xp: f32,
    pub gold: f32,
    pub it_dmg: f32,
    pub it_hp: f32,
    pub it_mp: f32,
    pub it_speed: f32,
    pub buys: [u32; 4],
    pub cds: [f32; 4],
    pub respawn: f32,
    pub dash: Option<Dash>,
    pub retreat: bool,
    pub think: f32,
    pub deaths: u32,
}

#[derive(Clone)]
pub struct Unit {
    pub id: u32,
    pub kind: Kind,
    pub team: usize,
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub hp: f32,
    pub max_hp: f32,
    pub dmg: f32,
    pub range: f32,
    pub interval: f32,
    pub speed: f32,
    pub proj: f32,
    pub atk_t: f32,
    pub stun: f32,
    pub slow: f32,
    pub slow_amt: f32,
    pub buff_t: f32,
    pub buff_as: f32,
    pub buff_ms: f32,
    pub hit: f32,
    pub swing: f32,
    pub inv_t: f32,
    pub retarget: f32,
    pub face: f32,
    pub dead: bool,
    pub tgt: Option<u32>,
    pub order: Order,
    pub ranged: bool,
    pub off_y: f32,
    pub idx: usize,
    pub hero: Option<HeroData>,
}

impl Unit {
    fn base(id: u32, kind: Kind, team: usize, x: f32, y: f32, r: f32, hp: f32) -> Unit {
        Unit {
            id,
            kind,
            team,
            x,
            y,
            r,
            hp,
            max_hp: hp,
            dmg: 0.0,
            range: 0.0,
            interval: 1.0,
            speed: 0.0,
            proj: 0.0,
            atk_t: 0.0,
            stun: 0.0,
            slow: 0.0,
            slow_amt: 1.0,
            buff_t: 0.0,
            buff_as: 1.0,
            buff_ms: 0.0,
            hit: 0.0,
            swing: 0.0,
            inv_t: 0.0,
            retarget: 0.0,
            face: if team == 0 { 0.0 } else { std::f32::consts::PI },
            dead: false,
            tgt: None,
            order: Order::None,
            ranged: false,
            off_y: 0.0,
            idx: 0,
            hero: None,
        }
    }
    pub fn is_struct(&self) -> bool {
        self.kind == Kind::Tower || self.kind == Kind::Throne
    }
}

pub struct Proj {
    pub id: u32,
    pub homing: bool,
    pub x: f32,
    pub y: f32,
    pub dx: f32,
    pub dy: f32,
    pub speed: f32,
    pub left: f32,
    pub width: f32,
    pub dmg: f32,
    pub src: u32,
    pub tgt: u32,
    pub hit: Vec<u32>,
    pub pierce: bool,
    pub slow: f32,
    pub slow_amt: f32,
    pub stun: f32,
    pub col: [f32; 3],
    pub done: bool,
}

pub struct Zone {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub r: f32,
    pub t: f32,
    pub delay: f32,
    pub dmg: f32,
    pub src: u32,
    pub stun: f32,
    pub slow: f32,
    pub slow_amt: f32,
    pub col: [f32; 3],
    pub done: bool,
}

/// События для слоя отображения.
pub enum Ev {
    Ring {
        x: f32,
        y: f32,
        r0: f32,
        r1: f32,
        dur: f32,
        col: [f32; 3],
    },
    Text {
        x: f32,
        y: f32,
        s: String,
        col: [f32; 3],
    },
    Feed {
        msg: String,
        team: usize,
    },
    End {
        winner: usize,
    },
}

struct Rng(u32);
impl Rng {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32) / (u32::MAX as f32)
    }
    fn range(&mut self, a: f32, b: f32) -> f32 {
        a + self.next() * (b - a)
    }
}

pub fn dist(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    ((ax - bx) * (ax - bx) + (ay - by) * (ay - by)).sqrt()
}
fn clampf(v: f32, a: f32, b: f32) -> f32 {
    v.max(a).min(b)
}
pub fn xp_need(lv: u32) -> f32 {
    110.0 + 70.0 * (lv as f32 - 1.0)
}

pub struct Game {
    pub t: f32,
    pub units: Vec<Unit>,
    pub projs: Vec<Proj>,
    pub zones: Vec<Zone>,
    pub events: Vec<Ev>,
    pub ruins: Vec<(f32, f32)>,
    pub wave: u32,
    pub wave_t: f32,
    pub kills: [u32; 2],
    pub diff: usize,
    pub player_id: u32,
    pub bot_id: u32,
    pub over: bool,
    pub winner: usize,
    pub rune: Option<(f32, f32)>,
    pub rune_t: f32,
    next_id: u32,
    rng: Rng,
}

impl Game {
    pub fn new(hero: usize, diff: usize, seed: u32) -> Game {
        let mut g = Game {
            t: 0.0,
            units: Vec::new(),
            projs: Vec::new(),
            zones: Vec::new(),
            events: Vec::new(),
            ruins: Vec::new(),
            wave: 0,
            wave_t: 3.0,
            kills: [0, 0],
            diff,
            player_id: 0,
            bot_id: 0,
            over: false,
            winner: 0,
            rune: None,
            rune_t: 45.0,
            next_id: 1,
            rng: Rng(seed.max(1)),
        };
        for team in 0..2usize {
            let bx = if team == 0 { 260.0 } else { W - 260.0 };
            let mut th = Unit::base(0, Kind::Throne, team, bx, LY, 58.0, 2600.0);
            th.idx = 0;
            g.mk(th);
            for (idx, off) in [1000.0f32, 640.0].iter().enumerate() {
                let x = if team == 0 { *off } else { W - *off };
                let mut t = Unit::base(0, Kind::Tower, team, x, LY, 36.0, 1500.0);
                t.dmg = 80.0;
                t.range = 520.0;
                t.interval = 1.15;
                t.proj = 1100.0;
                t.idx = idx;
                g.mk(t);
            }
        }
        let bot_key = (hero + 1 + (g.rng.next() * 2.0) as usize % 2) % 3;
        g.player_id = g.mk_hero(0, hero, false);
        g.bot_id = g.mk_hero(1, bot_key, true);
        g
    }

    fn mk(&mut self, mut u: Unit) -> u32 {
        u.id = self.next_id;
        self.next_id += 1;
        let id = u.id;
        self.units.push(u);
        id
    }

    fn mk_hero(&mut self, team: usize, key: usize, bot: bool) -> u32 {
        let d = &HEROES[key];
        let x = if team == 0 { 300.0 } else { W - 300.0 };
        let mut u = Unit::base(0, Kind::Hero, team, x, LY, d.r, 0.0);
        u.max_hp = 0.0;
        u.range = d.range;
        u.interval = d.interval;
        u.proj = d.proj;
        u.hero = Some(HeroData {
            key,
            bot,
            mp: 0.0,
            max_mp: 0.0,
            level: 1,
            xp: 0.0,
            gold: 220.0,
            it_dmg: 0.0,
            it_hp: 0.0,
            it_mp: 0.0,
            it_speed: 0.0,
            buys: [0; 4],
            cds: [0.0; 4],
            respawn: 0.0,
            dash: None,
            retreat: false,
            think: 0.0,
            deaths: 0,
        });
        let id = self.mk(u);
        let i = self.idx(id).unwrap();
        self.recalc(i);
        id
    }

    pub fn idx(&self, id: u32) -> Option<usize> {
        self.units.iter().position(|u| u.id == id)
    }
    pub fn player(&self) -> usize {
        self.idx(self.player_id).unwrap()
    }
    pub fn hd(&self, i: usize) -> &HeroData {
        self.units[i].hero.as_ref().unwrap()
    }
    fn hdm(&mut self, i: usize) -> &mut HeroData {
        self.units[i].hero.as_mut().unwrap()
    }

    fn recalc(&mut self, i: usize) {
        let (key, level, it_hp, it_mp, it_dmg, it_sp, bot) = {
            let h = self.hd(i);
            (
                h.key, h.level, h.it_hp, h.it_mp, h.it_dmg, h.it_speed, h.bot,
            )
        };
        let d = &HEROES[key];
        let l = level as f32 - 1.0;
        let (dm, hm) = if bot {
            (DIFFS[self.diff].dmg, DIFFS[self.diff].hp)
        } else {
            (1.0, 1.0)
        };
        let n_hp = d.hp * hm + l * 70.0 + it_hp;
        let n_mp = d.mp + l * 32.0 + it_mp;
        let max_hp = self.units[i].max_hp;
        let max_mp = self.hd(i).max_mp;
        self.units[i].hp += n_hp - max_hp;
        self.units[i].max_hp = n_hp;
        self.units[i].dmg = d.dmg * dm + l * 5.0 + it_dmg;
        self.units[i].speed = d.speed + it_sp;
        let h = self.hdm(i);
        h.mp += n_mp - max_mp;
        h.max_mp = n_mp;
    }

    pub fn item_cost(&self, i: usize, item: usize) -> f32 {
        (ITEM_BASE[item] * (1.0 + 0.22 * self.hd(i).buys[item] as f32)).round()
    }

    pub fn buy(&mut self, i: usize, item: usize) -> bool {
        let c = self.item_cost(i, item);
        if self.hd(i).gold < c {
            return false;
        }
        {
            let h = self.hdm(i);
            h.gold -= c;
            match item {
                0 => h.it_dmg += 12.0,
                1 => h.it_hp += 160.0,
                2 => h.it_speed += 22.0,
                _ => h.it_mp += 120.0,
            }
            h.buys[item] += 1;
        }
        self.recalc(i);
        true
    }

    // ---------- вспомогательные ----------
    pub fn invuln(&self, i: usize) -> bool {
        let u = &self.units[i];
        match u.kind {
            Kind::Tower => {
                u.idx == 1
                    && self
                        .units
                        .iter()
                        .any(|t| t.kind == Kind::Tower && t.team == u.team && t.idx == 0 && !t.dead)
            }
            Kind::Throne => self
                .units
                .iter()
                .any(|t| t.kind == Kind::Tower && t.team == u.team && !t.dead),
            _ => false,
        }
    }
    fn ring(&mut self, x: f32, y: f32, r0: f32, r1: f32, dur: f32, col: [f32; 3]) {
        self.events.push(Ev::Ring {
            x,
            y,
            r0,
            r1,
            dur,
            col,
        });
    }
    fn text(&mut self, x: f32, y: f32, s: String, col: [f32; 3]) {
        self.events.push(Ev::Text { x, y, s, col });
    }
    fn feed(&mut self, msg: String, team: usize) {
        self.events.push(Ev::Feed { msg, team });
    }
    fn in_range(&self, a: usize, b: usize) -> bool {
        let (ua, ub) = (&self.units[a], &self.units[b]);
        dist(ua.x, ua.y, ub.x, ub.y) <= ua.range + ua.r + ub.r
    }
    fn hero_color(&self, i: usize) -> [f32; 3] {
        match &self.units[i].hero {
            Some(h) => HEROES[h.key].color,
            None => TEAM_COL[self.units[i].team],
        }
    }

    fn apply_cc(&mut self, i: usize, stun: f32, slow: f32, slow_amt: f32) {
        let u = &mut self.units[i];
        if u.is_struct() {
            return;
        }
        if stun > 0.0 {
            u.stun = u.stun.max(stun);
        }
        if slow > 0.0 {
            u.slow = u.slow.max(slow);
            u.slow_amt = if slow_amt > 0.0 { slow_amt } else { 0.5 };
        }
    }

    pub fn deal_damage(&mut self, ti: usize, amt: f32, src: Option<usize>) {
        if self.units[ti].dead || self.over {
            return;
        }
        if self.invuln(ti) {
            if self.units[ti].inv_t <= 0.0 {
                self.units[ti].inv_t = 1.0;
                let (x, y, r) = (self.units[ti].x, self.units[ti].y, self.units[ti].r);
                self.text(x, y - r - 14.0, "Неуязвим".to_string(), [0.72, 0.78, 0.8]);
            }
            return;
        }
        self.units[ti].hp -= amt;
        self.units[ti].hit = 0.12;
        let src_hero = src.map_or(false, |s| self.units[s].kind == Kind::Hero);
        if self.units[ti].kind == Kind::Hero || src_hero {
            let ally = src.map_or(false, |s| self.units[s].team == 0);
            let col = if ally {
                [1.0, 0.95, 0.76]
            } else {
                [1.0, 0.61, 0.61]
            };
            let (x, y, r) = (self.units[ti].x, self.units[ti].y, self.units[ti].r);
            self.text(x, y - r - 18.0, format!("{}", amt.round() as i32), col);
        }
        if self.units[ti].hp <= 0.0 {
            self.kill(ti, src);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn aoe(
        &mut self,
        src: usize,
        x: f32,
        y: f32,
        r: f32,
        dmg: f32,
        stun: f32,
        slow: f32,
        slow_amt: f32,
    ) {
        let team = self.units[src].team;
        for j in 0..self.units.len() {
            let (dead, t, ux, uy, ur) = {
                let u = &self.units[j];
                (u.dead, u.team, u.x, u.y, u.r)
            };
            if dead || t == team {
                continue;
            }
            if dist(ux, uy, x, y) <= r + ur {
                if dmg > 0.0 {
                    self.deal_damage(j, dmg, Some(src));
                }
                if !self.units[j].dead {
                    self.apply_cc(j, stun, slow, slow_amt);
                }
            }
        }
        let col = self.hero_color(src);
        self.ring(x, y, r * 0.35, r, 0.45, col);
    }

    fn add_xp(&mut self, i: usize, xp: f32) {
        if self.hd(i).level >= 12 {
            return;
        }
        self.hdm(i).xp += xp;
        loop {
            let (lv, x) = (self.hd(i).level, self.hd(i).xp);
            if lv >= 12 || x < xp_need(lv) {
                break;
            }
            {
                let h = self.hdm(i);
                h.xp -= xp_need(lv);
                h.level += 1;
            }
            self.recalc(i);
            let (ux, uy, lvl, bot) = (
                self.units[i].x,
                self.units[i].y,
                self.hd(i).level,
                self.hd(i).bot,
            );
            self.ring(ux, uy, 10.0, 70.0, 0.6, [0.75, 0.55, 1.0]);
            self.text(ux, uy - 50.0, format!("Уровень {}", lvl), [0.85, 0.72, 1.0]);
            if !bot {
                self.feed(format!("Вы достигли уровня {}", lvl), 0);
            }
        }
    }

    fn kill(&mut self, ti: usize, src: Option<usize>) {
        let (kind, team, tx, ty) = {
            let u = &self.units[ti];
            (u.kind, u.team, u.x, u.y)
        };
        self.units[ti].dead = true;
        self.units[ti].hp = 0.0;
        let ot = 1 - team;
        let killer =
            src.filter(|&s| self.units[s].kind == Kind::Hero && self.units[s].team != team);
        let near: Vec<usize> = (0..self.units.len())
            .filter(|&j| {
                let h = &self.units[j];
                h.kind == Kind::Hero && h.team == ot && !h.dead && dist(h.x, h.y, tx, ty) < 800.0
            })
            .collect();
        match kind {
            Kind::Creep => {
                let ranged = self.units[ti].ranged;
                for &h in &near {
                    self.add_xp(h, if ranged { 56.0 } else { 42.0 });
                }
                if let Some(k) = killer {
                    let g = if ranged { 50.0 } else { 36.0 };
                    self.hdm(k).gold += g;
                    self.text(tx, ty - 26.0, format!("+{}", g as i32), [0.89, 0.70, 0.29]);
                }
            }
            Kind::Hero => {
                let lv = self.hd(ti).level;
                self.hdm(ti).deaths += 1;
                self.kills[ot] += 1;
                {
                    let h = self.hdm(ti);
                    h.respawn = 6.0 + lv as f32 * 2.2;
                    h.dash = None;
                }
                self.units[ti].order = Order::None;
                for &h in &near {
                    self.add_xp(h, 110.0 + 30.0 * lv as f32);
                }
                if let Some(k) = killer {
                    self.hdm(k).gold += 130.0 + 20.0 * lv as f32;
                }
                self.ring(tx, ty, 10.0, 130.0, 0.6, TEAM_COL[team]);
                let name = HEROES[self.hd(ti).key].name;
                let kn = killer.map(|k| HEROES[self.hd(k).key].name);
                self.feed(
                    match kn {
                        Some(k) => format!("{} повержен: {}", name, k),
                        None => format!("{} повержен", name),
                    },
                    ot,
                );
            }
            Kind::Tower => {
                for j in 0..self.units.len() {
                    if self.units[j].kind == Kind::Hero && self.units[j].team == ot {
                        self.hdm(j).gold += 170.0;
                    }
                }
                self.ring(tx, ty, 20.0, 260.0, 0.8, [1.0, 0.82, 0.48]);
                self.ruins.push((tx, ty));
                self.feed(format!("Башня «{}» разрушена", TEAM_NAME[team]), ot);
            }
            Kind::Throne => {
                self.ring(tx, ty, 30.0, 420.0, 1.2, [1.0, 0.82, 0.48]);
                self.over = true;
                self.winner = ot;
                self.events.push(Ev::End { winner: ot });
            }
        }
    }

    // ---------- бой ----------
    fn try_attack(&mut self, a: usize, tid: u32) -> bool {
        let t = match self.idx(tid) {
            Some(t) => t,
            None => return false,
        };
        if self.units[a].atk_t > 0.0 || self.units[a].stun > 0.0 || self.units[t].dead {
            return false;
        }
        let (tx, ty) = (self.units[t].x, self.units[t].y);
        let col = self.hero_color(a);
        {
            let u = &mut self.units[a];
            u.atk_t = u.interval / if u.buff_t > 0.0 { u.buff_as } else { 1.0 };
            u.face = (ty - u.y).atan2(tx - u.x);
            u.swing = 0.16;
        }
        let (ax, ay, dmg, proj, aid) = {
            let u = &self.units[a];
            (u.x, u.y, u.dmg, u.proj, u.id)
        };
        if proj > 0.0 {
            let id = self.next_id;
            self.next_id += 1;
            self.projs.push(Proj {
                id,
                homing: true,
                x: ax,
                y: ay,
                dx: 0.0,
                dy: 0.0,
                speed: proj,
                left: 0.0,
                width: 0.0,
                dmg,
                src: aid,
                tgt: tid,
                hit: Vec::new(),
                pierce: false,
                slow: 0.0,
                slow_amt: 1.0,
                stun: 0.0,
                col,
                done: false,
            });
        } else {
            self.deal_damage(t, dmg, Some(a));
        }
        true
    }

    pub fn cast(&mut self, h: usize, i: usize, tx: f32, ty: f32) -> bool {
        let (key, level, mp, cd, stun_h, dashing, dead) = {
            let u = &self.units[h];
            let d = u.hero.as_ref().unwrap();
            (
                d.key,
                d.level,
                d.mp,
                d.cds[i],
                u.stun,
                d.dash.is_some(),
                u.dead,
            )
        };
        let a = HEROES[key].abilities[i];
        if dead || stun_h > 0.0 || dashing || cd > 0.0 || mp < a.mana || self.over {
            return false;
        }
        let (hx, hy) = (self.units[h].x, self.units[h].y);
        let (dx, dy) = (tx - hx, ty - hy);
        let d = (dx * dx + dy * dy).sqrt().max(0.0001);
        let (ux, uy) = (dx / d, dy / d);
        let pw = 1.0 + 0.1 * (level as f32 - 1.0);
        self.units[h].face = uy.atan2(ux);
        let col = HEROES[key].color;
        match a.ty {
            AbType::SelfAoe => {
                self.aoe(h, hx, hy, a.radius, a.dmg * pw, a.stun, a.slow, a.slow_amt)
            }
            AbType::Buff => {
                let u = &mut self.units[h];
                u.buff_t = a.dur;
                u.buff_as = a.atk_speed;
                u.buff_ms = a.mv;
                self.ring(hx, hy, 10.0, 60.0, 0.5, [0.89, 0.70, 0.29]);
            }
            AbType::Skillshot => {
                let id = self.next_id;
                self.next_id += 1;
                let sid = self.units[h].id;
                self.projs.push(Proj {
                    id,
                    homing: false,
                    x: hx,
                    y: hy,
                    dx: ux,
                    dy: uy,
                    speed: a.speed,
                    left: a.range,
                    width: a.width,
                    dmg: a.dmg * pw,
                    src: sid,
                    tgt: 0,
                    hit: Vec::new(),
                    pierce: a.pierce,
                    slow: a.slow,
                    slow_amt: a.slow_amt,
                    stun: a.stun,
                    col,
                    done: false,
                });
            }
            AbType::AoePoint => {
                let k = d.min(a.range);
                let id = self.next_id;
                self.next_id += 1;
                let sid = self.units[h].id;
                self.zones.push(Zone {
                    id,
                    x: hx + ux * k,
                    y: hy + uy * k,
                    r: a.radius,
                    t: 0.0,
                    delay: a.delay.max(0.4),
                    dmg: a.dmg * pw,
                    src: sid,
                    stun: a.stun,
                    slow: a.slow,
                    slow_amt: a.slow_amt,
                    col,
                    done: false,
                });
            }
            AbType::Dash => {
                let k = d.min(a.range);
                let dash = Dash {
                    sx: hx,
                    sy: hy,
                    tx: clampf(hx + ux * k, 20.0, W - 20.0),
                    ty: clampf(hy + uy * k, 20.0, H - 20.0),
                    t: 0.0,
                    dur: 0.22,
                    ab: i,
                    pw,
                };
                self.hdm(h).dash = Some(dash);
                self.units[h].order = Order::None;
            }
            AbType::Blink => {
                let k = d.min(a.range);
                self.ring(hx, hy, 30.0, 6.0, 0.3, col);
                let nx = clampf(hx + ux * k, 20.0, W - 20.0);
                let ny = clampf(hy + uy * k, 20.0, H - 20.0);
                self.units[h].x = nx;
                self.units[h].y = ny;
                self.ring(nx, ny, 6.0, 40.0, 0.35, col);
            }
        }
        let hdm = self.hdm(h);
        hdm.mp -= a.mana;
        hdm.cds[i] = a.cd;
        true
    }

    // ---------- движение ----------
    fn move_toward(&mut self, i: usize, tx: f32, ty: f32, dt: f32) -> f32 {
        let u = &mut self.units[i];
        let (dx, dy) = (tx - u.x, ty - u.y);
        let d = (dx * dx + dy * dy).sqrt();
        if d < 2.0 {
            return d;
        }
        let sp = u.speed * if u.slow > 0.0 { u.slow_amt } else { 1.0 }
            + if u.buff_t > 0.0 { u.buff_ms } else { 0.0 };
        let st = d.min(sp * dt);
        u.x += dx / d * st;
        u.y += dy / d * st;
        u.face = dy.atan2(dx);
        u.x = clampf(u.x, u.r, W - u.r);
        u.y = clampf(u.y, u.r, H - u.r);
        d - st
    }

    fn nearest_enemy<F: Fn(&Unit) -> bool, M: Fn(&Unit) -> f32>(
        &self,
        team: usize,
        x: f32,
        y: f32,
        pred: F,
        maxd: M,
    ) -> Option<usize> {
        let mut best = None;
        let mut bd = 1e9f32;
        for (j, e) in self.units.iter().enumerate() {
            if e.dead || e.team == team || !pred(e) {
                continue;
            }
            let d = dist(x, y, e.x, e.y);
            if d <= maxd(e) && d < bd {
                bd = d;
                best = Some(j);
            }
        }
        best
    }

    fn find_target(&self, i: usize, rr: f32) -> Option<usize> {
        let u = &self.units[i];
        let t = self.nearest_enemy(u.team, u.x, u.y, |e| !e.is_struct(), |e| rr + e.r);
        if t.is_some() {
            return t;
        }
        // сооружения: неуязвимость проверяем отдельно
        let mut best = None;
        let mut bd = 1e9f32;
        for (j, e) in self.units.iter().enumerate() {
            if e.dead || e.team == u.team || !e.is_struct() || self.invuln(j) {
                continue;
            }
            let d = dist(u.x, u.y, e.x, e.y);
            if d <= u.range + e.r + 60.0 && d < bd {
                bd = d;
                best = Some(j);
            }
        }
        best
    }

    fn tick_timers(&mut self, i: usize, dt: f32) {
        let u = &mut self.units[i];
        for t in [
            &mut u.atk_t,
            &mut u.stun,
            &mut u.slow,
            &mut u.buff_t,
            &mut u.hit,
            &mut u.swing,
            &mut u.retarget,
            &mut u.inv_t,
        ] {
            if *t > 0.0 {
                *t -= dt;
            }
        }
    }

    fn update_creep(&mut self, i: usize, dt: f32) {
        if self.units[i].stun > 0.0 {
            return;
        }
        let (ux, uy) = (self.units[i].x, self.units[i].y);
        let mut t = self.units[i].tgt.and_then(|id| self.idx(id)).filter(|&j| {
            let e = &self.units[j];
            !e.dead && !self.invuln(j) && dist(ux, uy, e.x, e.y) <= 520.0
        });
        if t.is_none() || self.units[i].retarget <= 0.0 {
            t = self.find_target(i, 380.0);
            self.units[i].retarget = 0.4;
        }
        self.units[i].tgt = t.map(|j| self.units[j].id);
        match t {
            Some(j) => {
                let (tid, tx, ty) = (self.units[j].id, self.units[j].x, self.units[j].y);
                if self.in_range(i, j) {
                    self.units[i].face = (ty - uy).atan2(tx - ux);
                    self.try_attack(i, tid);
                } else {
                    self.move_toward(i, tx, ty, dt);
                }
            }
            None => {
                let team = self.units[i].team;
                let throne = self
                    .units
                    .iter()
                    .find(|e| e.kind == Kind::Throne && e.team == 1 - team)
                    .map(|e| e.x)
                    .unwrap_or(W / 2.0);
                let oy = self.units[i].off_y;
                self.move_toward(i, throne, LY + oy, dt);
            }
        }
    }

    fn update_tower(&mut self, i: usize) {
        if self.units[i].stun > 0.0 {
            return;
        }
        let (team, x, y, range) = {
            let u = &self.units[i];
            (u.team, u.x, u.y, u.range)
        };
        let mut t = self.units[i].tgt.and_then(|id| self.idx(id)).filter(|&j| {
            let e = &self.units[j];
            !e.dead && dist(x, y, e.x, e.y) <= range + e.r
        });
        if t.is_none() {
            t = self.nearest_enemy(team, x, y, |e| e.kind == Kind::Creep, |e| range + e.r);
        }
        if t.is_none() {
            t = self.nearest_enemy(team, x, y, |e| e.kind == Kind::Hero, |e| range + e.r);
        }
        self.units[i].tgt = t.map(|j| self.units[j].id);
        if let Some(j) = t {
            let tid = self.units[j].id;
            self.try_attack(i, tid);
        }
    }

    fn exec_order(&mut self, h: usize, dt: f32) {
        match self.units[h].order {
            Order::None => {
                let u = &self.units[h];
                let t = self.nearest_enemy(
                    u.team,
                    u.x,
                    u.y,
                    |e| !e.is_struct(),
                    |e| u.range + u.r + e.r,
                );
                if let Some(j) = t {
                    let tid = self.units[j].id;
                    self.try_attack(h, tid);
                }
            }
            Order::Move(x, y) => {
                if self.move_toward(h, x, y, dt) < 4.0 {
                    self.units[h].order = Order::None;
                }
            }
            Order::Attack(tid) => match self.idx(tid) {
                Some(j) if !self.units[j].dead && !self.invuln(j) => {
                    if self.in_range(h, j) {
                        let (tx, ty) = (self.units[j].x, self.units[j].y);
                        let (hx, hy) = (self.units[h].x, self.units[h].y);
                        self.units[h].face = (ty - hy).atan2(tx - hx);
                        self.try_attack(h, tid);
                    } else {
                        let (tx, ty) = (self.units[j].x, self.units[j].y);
                        self.move_toward(h, tx, ty, dt);
                    }
                }
                _ => self.units[h].order = Order::None,
            },
        }
    }

    fn update_hero(&mut self, h: usize, dt: f32) {
        let team = self.units[h].team;
        if self.units[h].dead {
            self.hdm(h).respawn -= dt;
            if self.hd(h).respawn <= 0.0 {
                let max_hp = self.units[h].max_hp;
                let u = &mut self.units[h];
                u.dead = false;
                u.hp = max_hp;
                u.x = if team == 0 { 300.0 } else { W - 300.0 };
                u.y = LY;
                u.order = Order::None;
                u.stun = 0.0;
                u.slow = 0.0;
                u.buff_t = 0.0;
                let (x, y) = (u.x, u.y);
                let hd = self.hdm(h);
                hd.mp = hd.max_mp;
                hd.dash = None;
                hd.retreat = false;
                self.ring(x, y, 10.0, 120.0, 0.6, TEAM_COL[team]);
            }
            return;
        }
        let (max_hp, max_mp) = (self.units[h].max_hp, self.hd(h).max_mp);
        self.units[h].hp = (self.units[h].hp + (max_hp * 0.004 + 1.5) * dt).min(max_hp);
        {
            let hd = self.hdm(h);
            hd.mp = (hd.mp + (1.8 + max_mp * 0.008) * dt).min(max_mp);
            for c in hd.cds.iter_mut() {
                if *c > 0.0 {
                    *c = (*c - dt).max(0.0);
                }
            }
        }
        let (home_x, home_y) = if team == 0 {
            (260.0, LY)
        } else {
            (W - 260.0, LY)
        };
        if dist(self.units[h].x, self.units[h].y, home_x, home_y) < 290.0 {
            self.units[h].hp = (self.units[h].hp + max_hp * 0.14 * dt).min(max_hp);
            let hd = self.hdm(h);
            hd.mp = (hd.mp + max_mp * 0.14 * dt).min(max_mp);
        }
        if let Some(mut d) = self.hd(h).dash {
            d.t += dt;
            let k = (d.t / d.dur).min(1.0);
            self.units[h].x = d.sx + (d.tx - d.sx) * k;
            self.units[h].y = d.sy + (d.ty - d.sy) * k;
            if k >= 1.0 {
                let a = HEROES[self.hd(h).key].abilities[d.ab];
                let (x, y) = (self.units[h].x, self.units[h].y);
                if a.dmg > 0.0 {
                    self.aoe(h, x, y, a.radius, a.dmg * d.pw, a.stun, a.slow, a.slow_amt);
                } else {
                    let col = self.hero_color(h);
                    self.ring(x, y, 6.0, 44.0, 0.3, col);
                }
                self.hdm(h).dash = None;
            } else {
                self.hdm(h).dash = Some(d);
            }
            return;
        }
        if self.units[h].stun > 0.0 {
            return;
        }
        if self.hd(h).bot {
            self.bot_think(h, dt);
        }
        self.exec_order(h, dt);
    }

    // ---------- ИИ героя ----------
    fn count_enemies(&self, team: usize, x: f32, y: f32, r: f32) -> usize {
        self.units
            .iter()
            .filter(|e| {
                !e.dead && e.team != team && !e.is_struct() && dist(e.x, e.y, x, y) <= r + e.r
            })
            .count()
    }
    fn tanked_by(&self, h: usize) -> bool {
        let u = &self.units[h];
        self.units
            .iter()
            .filter(|c| {
                !c.dead
                    && c.kind == Kind::Creep
                    && c.team == u.team
                    && dist(c.x, c.y, u.x, u.y) < 380.0
            })
            .count()
            >= 2
    }
    fn under_enemy_tower(&self, x: f32, y: f32, team: usize) -> bool {
        self.units.iter().any(|t| {
            t.kind == Kind::Tower
                && t.team != team
                && !t.dead
                && dist(t.x, t.y, x, y) <= t.range + 70.0
        })
    }

    fn bot_think(&mut self, h: usize, dt: f32) {
        self.hdm(h).think -= dt;
        if self.hd(h).think > 0.0 {
            return;
        }
        let think = DIFFS[self.diff].think;
        self.hdm(h).think = 0.28 / think;
        let team = self.units[h].team;
        let s = if team == 0 { 1.0f32 } else { -1.0 };
        let home_x = if team == 0 { 260.0 } else { W - 260.0 };
        let foe = match self
            .units
            .iter()
            .position(|u| u.kind == Kind::Hero && u.team != team)
        {
            Some(f) => f,
            None => return,
        };
        // покупки
        let mut pick: Option<usize> = None;
        for i in 0..4 {
            if self.hd(h).gold >= self.item_cost(h, i)
                && pick.map_or(true, |p| self.hd(h).buys[i] < self.hd(h).buys[p])
            {
                pick = Some(i);
            }
        }
        if let Some(p) = pick {
            if !self.hd(h).retreat {
                self.buy(h, p);
            }
        }
        // отступление
        if self.hd(h).retreat {
            if self.units[h].hp > self.units[h].max_hp * 0.85 {
                self.hdm(h).retreat = false;
            } else {
                self.units[h].order = Order::Move(home_x + s * 90.0, LY);
                self.bot_cast(h, foe, true);
                return;
            }
        } else if self.units[h].hp < self.units[h].max_hp * 0.3
            && (self.units[h].x - home_x).abs() > 300.0
        {
            self.hdm(h).retreat = true;
            return;
        }
        let (hx, hy, hr, hrange, hdmg) = {
            let u = &self.units[h];
            (u.x, u.y, u.r, u.range, u.dmg)
        };
        let tanked = self.tanked_by(h);
        // руна
        if let Some((rx, ry)) = self.rune {
            let f = &self.units[foe];
            if dist(hx, hy, rx, ry) < 800.0 && !f.dead && dist(hx, hy, f.x, f.y) > 600.0 {
                self.units[h].order = Order::Move(rx, ry);
                return;
            }
        }
        let mut best: Option<usize> = None;
        let mut bs = -1e9f32;
        for j in 0..self.units.len() {
            let e = &self.units[j];
            if e.dead || e.team == team || self.invuln(j) {
                continue;
            }
            let d = dist(hx, hy, e.x, e.y);
            if d > hrange + e.r + 170.0 {
                continue;
            }
            if !tanked && self.under_enemy_tower(e.x, e.y, team) {
                continue;
            }
            let mut sc = -d;
            match e.kind {
                Kind::Hero => sc += 250.0 + (1.0 - e.hp / e.max_hp) * 400.0,
                Kind::Creep => sc += if e.hp <= hdmg { 220.0 } else { 0.0 },
                _ => {
                    if !tanked {
                        continue;
                    }
                    sc -= 100.0;
                }
            }
            if sc > bs {
                bs = sc;
                best = Some(j);
            }
        }
        let _ = hr;
        if let Some(j) = best {
            self.units[h].order = Order::Attack(self.units[j].id);
        } else {
            let mine: Vec<f32> = self
                .units
                .iter()
                .filter(|c| !c.dead && c.kind == Kind::Creep && c.team == team)
                .map(|c| c.x)
                .collect();
            let mut x;
            if !mine.is_empty() {
                x = if s > 0.0 {
                    mine.iter().cloned().fold(f32::MIN, f32::max)
                } else {
                    mine.iter().cloned().fold(f32::MAX, f32::min)
                };
                x -= s * 70.0;
            } else {
                let tx = self
                    .units
                    .iter()
                    .find(|t| t.kind == Kind::Tower && t.team == team && t.idx == 0 && !t.dead)
                    .map(|t| t.x)
                    .unwrap_or(home_x);
                x = tx - s * 60.0;
            }
            let oy = self.rng.range(-40.0, 40.0);
            self.units[h].order = Order::Move(clampf(x, 100.0, W - 100.0), LY + oy);
        }
        self.bot_cast(h, foe, false);
    }

    fn bot_cast(&mut self, h: usize, foe: usize, fleeing: bool) {
        let (hx, hy, team) = (self.units[h].x, self.units[h].y, self.units[h].team);
        let s = if team == 0 { 1.0f32 } else { -1.0 };
        let (fx, fy, falive, fhp) = {
            let f = &self.units[foe];
            (f.x, f.y, !f.dead, f.hp / f.max_hp)
        };
        let key = self.hd(h).key;
        for i in 0..4 {
            let a = HEROES[key].abilities[i];
            if self.hd(h).cds[i] > 0.0 || self.hd(h).mp < a.mana {
                continue;
            }
            let dfoe = dist(hx, hy, fx, fy);
            match a.ty {
                AbType::SelfAoe => {
                    let hit_foe = falive && dfoe < a.radius + self.units[foe].r;
                    let ok = if a.key == 'R' {
                        hit_foe
                    } else {
                        self.count_enemies(team, hx, hy, a.radius) >= 2 || hit_foe
                    };
                    if ok {
                        self.cast(h, i, hx, hy);
                        return;
                    }
                }
                AbType::Buff => {
                    if self.count_enemies(team, hx, hy, 450.0) > 0 || (falive && dfoe < 600.0) {
                        self.cast(h, i, hx, hy);
                        return;
                    }
                }
                AbType::Skillshot => {
                    if falive && dfoe < a.range * 0.95 && (a.key != 'R' || fhp < 0.75) {
                        self.cast(h, i, fx, fy);
                        return;
                    }
                }
                AbType::AoePoint => {
                    if falive && dfoe < a.range && (a.key != 'R' || fhp < 0.8) {
                        self.cast(h, i, fx, fy);
                        return;
                    }
                    if a.key != 'R' {
                        let mut spot: Option<(f32, f32)> = None;
                        for e in self.units.iter() {
                            if e.dead
                                || e.team == team
                                || e.is_struct()
                                || dist(hx, hy, e.x, e.y) > a.range
                            {
                                continue;
                            }
                            if self.count_enemies(team, e.x, e.y, a.radius) >= 3 {
                                spot = Some((e.x, e.y));
                                break;
                            }
                        }
                        if let Some((sx, sy)) = spot {
                            self.cast(h, i, sx, sy);
                            return;
                        }
                    }
                }
                AbType::Dash | AbType::Blink => {
                    if fleeing {
                        self.cast(h, i, hx - s * 300.0, LY);
                        return;
                    }
                    if a.ty == AbType::Dash
                        && a.dmg > 0.0
                        && falive
                        && dfoe < a.range + a.radius * 0.5
                        && dfoe > 140.0
                    {
                        self.cast(h, i, fx, fy);
                        return;
                    }
                }
            }
        }
    }

    // ---------- крипы ----------
    fn spawn_wave(&mut self) {
        self.wave += 1;
        let sc = 1.0 + 0.04 * (self.wave as f32 - 1.0);
        for team in 0..2usize {
            let s = if team == 0 { 1.0f32 } else { -1.0 };
            let bx = if team == 0 { 430.0 } else { W - 430.0 };
            for i in 0..4 {
                let ranged = i == 3;
                let r = if ranged { 12.0 } else { 15.0 };
                let jitter = self.rng.range(-8.0, 8.0);
                let off = self.rng.range(-60.0, 60.0);
                let mut u = Unit::base(
                    0,
                    Kind::Creep,
                    team,
                    bx - s * i as f32 * 38.0,
                    LY + if i % 2 == 1 { 18.0 } else { -18.0 } + jitter,
                    r,
                    (if ranged { 210.0 } else { 330.0 }) * sc,
                );
                u.dmg = (if ranged { 27.0 } else { 20.0 }) * sc;
                u.range = if ranged { 250.0 } else { 38.0 };
                u.interval = if ranged { 1.35 } else { 1.1 };
                u.speed = if ranged { 92.0 } else { 98.0 };
                u.proj = if ranged { 650.0 } else { 0.0 };
                u.ranged = ranged;
                u.off_y = off;
                self.mk(u);
            }
        }
    }

    fn separate(&mut self) {
        let n = self.units.len();
        for a in 0..n {
            if self.units[a].dead || self.units[a].is_struct() || self.hero_dashing(a) {
                continue;
            }
            for b in (a + 1)..n {
                if self.units[b].dead || self.units[b].is_struct() || self.hero_dashing(b) {
                    continue;
                }
                let (ax, ay, ar) = (self.units[a].x, self.units[a].y, self.units[a].r);
                let (bx, by, br) = (self.units[b].x, self.units[b].y, self.units[b].r);
                let (dx, dy) = (bx - ax, by - ay);
                let d = (dx * dx + dy * dy).sqrt().max(0.01);
                let min = ar + br;
                if d < min {
                    let p = (min - d) / 2.0;
                    let (nx, ny) = (dx / d, dy / d);
                    self.units[a].x -= nx * p;
                    self.units[a].y -= ny * p;
                    self.units[b].x += nx * p;
                    self.units[b].y += ny * p;
                }
            }
            for s in 0..n {
                if self.units[s].dead || !self.units[s].is_struct() {
                    continue;
                }
                let (sx, sy, sr) = (self.units[s].x, self.units[s].y, self.units[s].r);
                let (ax, ay, ar) = (self.units[a].x, self.units[a].y, self.units[a].r);
                let (dx, dy) = (ax - sx, ay - sy);
                let d = (dx * dx + dy * dy).sqrt().max(0.01);
                let min = ar + sr;
                if d < min {
                    self.units[a].x = sx + dx / d * min;
                    self.units[a].y = sy + dy / d * min;
                }
            }
            let r = self.units[a].r;
            self.units[a].x = clampf(self.units[a].x, r, W - r);
            self.units[a].y = clampf(self.units[a].y, r, H - r);
        }
    }

    fn hero_dashing(&self, i: usize) -> bool {
        self.units[i]
            .hero
            .as_ref()
            .map_or(false, |h| h.dash.is_some())
    }

    // ---------- главный шаг ----------
    pub fn update(&mut self, dt: f32) {
        if self.over {
            return;
        }
        self.t += dt;
        self.wave_t -= dt;
        if self.wave_t <= 0.0 {
            self.spawn_wave();
            self.wave_t = 24.0;
        }
        self.rune_t -= dt;
        if self.rune.is_none() && self.rune_t <= 0.0 {
            let sign = if self.rng.next() < 0.5 { -1.0 } else { 1.0 };
            let off = self.rng.range(50.0, 140.0);
            self.rune = Some((W / 2.0, LY + sign * off));
            self.feed("В центре карты появилась руна".to_string(), 0);
        }
        let gm = DIFFS[self.diff].gold;
        for i in 0..self.units.len() {
            if self.units[i].kind == Kind::Hero {
                let bot = self.hd(i).bot;
                self.hdm(i).gold += dt * 1.7 * if bot { gm } else { 1.0 };
            }
        }
        for i in 0..self.units.len() {
            if self.units[i].dead && self.units[i].kind != Kind::Hero {
                continue;
            }
            self.tick_timers(i, dt);
            match self.units[i].kind {
                Kind::Hero => self.update_hero(i, dt),
                Kind::Creep => self.update_creep(i, dt),
                Kind::Tower => self.update_tower(i),
                Kind::Throne => {}
            }
            if self.over {
                return;
            }
        }
        if let Some((rx, ry)) = self.rune {
            for i in 0..self.units.len() {
                let u = &self.units[i];
                if u.kind == Kind::Hero && !u.dead && dist(u.x, u.y, rx, ry) < u.r + 28.0 {
                    let (max_hp, name, team) = (u.max_hp, HEROES[self.hd(i).key].name, u.team);
                    self.units[i].hp = (self.units[i].hp + max_hp * 0.4).min(max_hp);
                    self.units[i].buff_t = 8.0;
                    self.units[i].buff_as = 1.3;
                    self.units[i].buff_ms = 70.0;
                    let hd = self.hdm(i);
                    hd.mp = (hd.mp + hd.max_mp * 0.4).min(hd.max_mp);
                    hd.gold += 120.0;
                    self.ring(rx, ry, 10.0, 120.0, 0.6, [0.89, 0.70, 0.29]);
                    self.feed(format!("{} забирает руну", name), team);
                    self.rune = None;
                    self.rune_t = 60.0;
                    break;
                }
            }
        }
        self.separate();
        self.update_projs(dt);
        self.update_zones(dt);
        self.units.retain(|u| !u.dead || u.kind == Kind::Hero);
    }

    fn update_projs(&mut self, dt: f32) {
        for p in 0..self.projs.len() {
            if self.projs[p].homing {
                let tid = self.projs[p].tgt;
                let sid = self.projs[p].src;
                let (tj, sj) = (self.idx(tid), self.idx(sid));
                match tj {
                    Some(tj) if !self.units[tj].dead => {
                        let (tx, ty, tr) = (self.units[tj].x, self.units[tj].y, self.units[tj].r);
                        let (px, py, sp) = (self.projs[p].x, self.projs[p].y, self.projs[p].speed);
                        let (dx, dy) = (tx - px, ty - py);
                        let d = (dx * dx + dy * dy).sqrt().max(0.0001);
                        let st = sp * dt;
                        if d <= st + tr * 0.5 {
                            let dmg = self.projs[p].dmg;
                            self.deal_damage(tj, dmg, sj);
                            self.projs[p].done = true;
                        } else {
                            self.projs[p].x += dx / d * st;
                            self.projs[p].y += dy / d * st;
                        }
                    }
                    _ => self.projs[p].done = true,
                }
            } else {
                let (dx, dy, sp) = (self.projs[p].dx, self.projs[p].dy, self.projs[p].speed);
                let st = sp * dt;
                self.projs[p].x += dx * st;
                self.projs[p].y += dy * st;
                self.projs[p].left -= st;
                let (px, py, w, sid) = (
                    self.projs[p].x,
                    self.projs[p].y,
                    self.projs[p].width,
                    self.projs[p].src,
                );
                let team = self.idx(sid).map_or(0, |s| self.units[s].team);
                let sj = self.idx(sid);
                for j in 0..self.units.len() {
                    let (dead, t, ux, uy, ur, uid) = {
                        let u = &self.units[j];
                        (u.dead, u.team, u.x, u.y, u.r, u.id)
                    };
                    if dead || t == team || self.projs[p].hit.contains(&uid) {
                        continue;
                    }
                    if dist(ux, uy, px, py) <= ur + w {
                        self.projs[p].hit.push(uid);
                        let (dmg, stun, slow, sa) = (
                            self.projs[p].dmg,
                            self.projs[p].stun,
                            self.projs[p].slow,
                            self.projs[p].slow_amt,
                        );
                        self.deal_damage(j, dmg, sj);
                        if !self.units[j].dead {
                            self.apply_cc(j, stun, slow, sa);
                        }
                        if !self.projs[p].pierce {
                            self.projs[p].done = true;
                            break;
                        }
                    }
                }
                if self.projs[p].left <= 0.0 {
                    self.projs[p].done = true;
                }
            }
        }
        self.projs.retain(|p| !p.done);
    }

    fn update_zones(&mut self, dt: f32) {
        for z in 0..self.zones.len() {
            self.zones[z].t += dt;
            if self.zones[z].t >= self.zones[z].delay {
                let (x, y, r, dmg, st, sl, sa, sid) = {
                    let q = &self.zones[z];
                    (q.x, q.y, q.r, q.dmg, q.stun, q.slow, q.slow_amt, q.src)
                };
                if let Some(s) = self.idx(sid) {
                    self.aoe(s, x, y, r, dmg, st, sl, sa);
                }
                self.zones[z].done = true;
            }
        }
        self.zones.retain(|z| !z.done);
    }
}
