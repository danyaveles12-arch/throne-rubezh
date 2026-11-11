//! Тронный рубеж: мини-MOBA на Bevy 0.14.
//! Логика игры лежит в sim.rs, здесь только 3D-сцена, мышь и интерфейс.

mod sim;

use bevy::input::mouse::MouseWheel;
use bevy::pbr::CascadeShadowConfigBuilder;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use sim::*;
use std::collections::HashMap;

/// Масштаб: одна единица логики = 0.1 единицы сцены.
const S: f32 = 0.1;
const PITCH: f32 = 1.0;

fn wp(x: f32, y: f32) -> Vec3 {
    Vec3::new(x * S, 0.0, y * S)
}
fn col(c: [f32; 3]) -> Color {
    Color::srgb(c[0], c[1], c[2])
}
fn hex(h: u32) -> [f32; 3] {
    [
        ((h >> 16) & 255) as f32 / 255.0,
        ((h >> 8) & 255) as f32 / 255.0,
        (h & 255) as f32 / 255.0,
    ]
}

// ---------- компоненты и ресурсы ----------
#[derive(Component)]
struct MainCam;
#[derive(Component)]
struct UnitView;
#[derive(Component)]
struct ProjView;
#[derive(Component)]
struct ZoneView;
#[derive(Component)]
struct Spin;
#[derive(Component)]
struct Shield(u32);
#[derive(Component)]
struct Fade {
    t: f32,
    dur: f32,
    r0: f32,
    r1: f32,
    rgb: [f32; 3],
}
#[derive(Component)]
struct HpBar {
    id: u32,
    w: f32,
    up: f32,
}
#[derive(Component)]
struct HpFill;
#[derive(Component)]
struct FloatText {
    pos: Vec3,
    t: f32,
    rgb: [f32; 3],
}
#[derive(Component)]
struct FeedItem {
    ttl: f32,
}
#[derive(Component)]
struct FeedBox;
#[derive(Component)]
enum UiTag {
    Top,
    Hud,
    EndTitle,
    EndStats,
}
#[derive(Component)]
enum UiBar {
    Hp,
    Mp,
    Xp,
}
#[derive(Component)]
struct MenuRoot;
#[derive(Component)]
struct EndRoot;
#[derive(Component)]
struct HeroBtn(usize);
#[derive(Component)]
struct DiffBtn(usize);
#[derive(Component)]
struct StartBtn;
#[derive(Component)]
struct AgainBtn;

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Menu,
    Playing,
    Paused,
    Over,
}

#[derive(Resource)]
struct UiFont(Handle<Font>);

#[derive(Resource)]
struct Gfx {
    cube: Handle<Mesh>,
    sphere: Handle<Mesh>,
    cyl: Handle<Mesh>,
}

#[derive(Resource, Default)]
struct Views {
    units: HashMap<u32, Entity>,
    projs: HashMap<u32, Entity>,
    zones: HashMap<u32, Entity>,
    bars: HashMap<u32, Entity>,
    ruins: Vec<Entity>,
}

#[derive(Resource)]
struct Session {
    game: Game,
    screen: Screen,
    hero: usize,
    diff: usize,
    aim: Option<usize>,
    focus: Vec2,
    dist: f32,
    locked: bool,
    mouse: Vec2,
    hover: Option<u32>,
    hold_t: f32,
    edge_t: f32,
}

fn seed_now() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(12345)
        | 1
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Тронный рубеж".into(),
                resolution: (1280.0, 800.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.04, 0.07, 0.08)))
        .insert_resource(AmbientLight {
            color: Color::srgb(0.8, 0.9, 1.0),
            brightness: 420.0,
        })
        .insert_resource(Views::default())
        .insert_resource(Session {
            game: Game::new(0, 1, seed_now()),
            screen: Screen::Menu,
            hero: 0,
            diff: 1,
            aim: None,
            focus: Vec2::new(300.0, LY),
            dist: 82.0,
            locked: true,
            mouse: Vec2::new(300.0, LY),
            hover: None,
            hold_t: 0.0,
            edge_t: 0.0,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                menu_system,
                input_system,
                step_game,
                consume_events,
                sync_views,
                camera_system,
                update_bars,
                update_hud,
                end_screen,
                fade_system,
                float_text_system,
                feed_system,
                spin_system,
                draw_gizmos,
            )
                .chain(),
        )
        .run();
}

// ---------- материалы ----------
fn solid(mats: &mut Assets<StandardMaterial>, c: [f32; 3]) -> Handle<StandardMaterial> {
    mats.add(StandardMaterial {
        base_color: col(c),
        perceptual_roughness: 0.85,
        ..default()
    })
}
fn glowm(mats: &mut Assets<StandardMaterial>, c: [f32; 3]) -> Handle<StandardMaterial> {
    mats.add(StandardMaterial {
        base_color: col(c),
        unlit: true,
        ..default()
    })
}
fn ghost(mats: &mut Assets<StandardMaterial>, c: [f32; 3], a: f32) -> Handle<StandardMaterial> {
    mats.add(StandardMaterial {
        base_color: Color::srgba(c[0], c[1], c[2], a),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    })
}

fn part(
    p: &mut ChildBuilder,
    mesh: &Handle<Mesh>,
    mat: &Handle<StandardMaterial>,
    scale: Vec3,
    pos: Vec3,
) -> Entity {
    p.spawn(PbrBundle {
        mesh: mesh.clone(),
        material: mat.clone(),
        transform: Transform {
            translation: pos,
            scale,
            ..default()
        },
        ..default()
    })
    .id()
}

fn text_bundle(font: &Handle<Font>, s: &str, size: f32, color: Color) -> TextBundle {
    TextBundle::from_section(
        s.to_string(),
        TextStyle {
            font: font.clone(),
            font_size: size,
            color,
        },
    )
}

// ---------- запуск сцены и интерфейса ----------
fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let font: Handle<Font> = assets.load("fonts/DejaVuSans.ttf");
    commands.insert_resource(UiFont(font.clone()));
    let gfx = Gfx {
        cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        sphere: meshes.add(Sphere::new(1.0).mesh().uv(24, 16)),
        cyl: meshes.add(Cylinder::new(1.0, 1.0)),
    };

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(30.0, 70.0, 95.0)
                .looking_at(Vec3::new(30.0, 0.0, 50.0), Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 38f32.to_radians(),
                ..default()
            }),
            ..default()
        },
        MainCam,
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 9000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(-60.0, 120.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 2,
            maximum_distance: 220.0,
            first_cascade_far_bound: 70.0,
            ..default()
        }
        .into(),
        ..default()
    });

    // земля и линия
    let ground = meshes.add(Plane3d::default().mesh().size(W * S + 400.0, H * S + 400.0));
    commands.spawn(PbrBundle {
        mesh: ground,
        material: solid(&mut mats, hex(0x2a4d3a)),
        transform: Transform::from_xyz(W * S / 2.0, 0.0, H * S / 2.0),
        ..default()
    });
    let lane = meshes.add(Plane3d::default().mesh().size(W * S, 27.0));
    commands.spawn(PbrBundle {
        mesh: lane,
        material: solid(&mut mats, hex(0x6b5b43)),
        transform: Transform::from_xyz(W * S / 2.0, 0.08, LY * S),
        ..default()
    });
    // подсвеченные базы и центр
    for t in 0..2usize {
        let bx = if t == 0 { 260.0 } else { W - 260.0 };
        commands.spawn(PbrBundle {
            mesh: gfx.cyl.clone(),
            material: ghost(&mut mats, TEAM_COL[t], 0.22),
            transform: Transform {
                translation: Vec3::new(bx * S, 0.14, LY * S),
                scale: Vec3::new(29.0, 0.02, 29.0),
                ..default()
            },
            ..default()
        });
    }
    commands.spawn(PbrBundle {
        mesh: gfx.cyl.clone(),
        material: ghost(&mut mats, hex(0xe2b24a), 0.18),
        transform: Transform {
            translation: Vec3::new(W * S / 2.0, 0.14, LY * S),
            scale: Vec3::new(11.0, 0.02, 11.0),
            ..default()
        },
        ..default()
    });

    // лес
    let trunk_m = solid(&mut mats, hex(0x5a3f28));
    let leaf = [
        solid(&mut mats, hex(0x1c4a33)),
        solid(&mut mats, hex(0x235a3c)),
        solid(&mut mats, hex(0x2c6a44)),
    ];
    let mut seed: u32 = 20261106;
    let mut rnd = move || {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        (seed >> 8) as f32 / 16777216.0
    };
    for _ in 0..280 {
        let x = rnd() * W;
        let y = rnd() * H;
        let r = 16.0 + rnd() * 22.0;
        let k = (rnd() * 3.0) as usize % 3;
        if (y - LY).abs() < 170.0 || ((x < 560.0 || x > W - 560.0) && (y - LY).abs() < 360.0) {
            continue;
        }
        let s = r / 30.0;
        commands.spawn(PbrBundle {
            mesh: gfx.cyl.clone(),
            material: trunk_m.clone(),
            transform: Transform {
                translation: Vec3::new(x * S, 0.9 * s, y * S),
                scale: Vec3::new(0.35 * s, 1.8 * s, 0.35 * s),
                ..default()
            },
            ..default()
        });
        commands.spawn(PbrBundle {
            mesh: gfx.sphere.clone(),
            material: leaf[k].clone(),
            transform: Transform {
                translation: Vec3::new(x * S, 4.2 * s, y * S),
                scale: Vec3::new(r * S * 1.1, 4.2 * s, r * S * 1.1),
                ..default()
            },
            ..default()
        });
        commands.spawn(PbrBundle {
            mesh: gfx.sphere.clone(),
            material: leaf[(k + 1) % 3].clone(),
            transform: Transform {
                translation: Vec3::new(x * S, 6.2 * s, y * S),
                scale: Vec3::new(r * S * 0.75, 3.4 * s, r * S * 0.75),
                ..default()
            },
            ..default()
        });
    }
    commands.insert_resource(gfx);

    build_ui(&mut commands, &font);
}

fn ui_bar(p: &mut ChildBuilder, tag: UiBar, color: Color, h: f32) {
    p.spawn(NodeBundle {
        style: Style {
            width: Val::Percent(100.0),
            height: Val::Px(h),
            ..default()
        },
        background_color: Color::srgb(0.04, 0.07, 0.09).into(),
        ..default()
    })
    .with_children(|b| {
        b.spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                background_color: color.into(),
                ..default()
            },
            tag,
        ));
    });
}

fn button(p: &mut ChildBuilder, font: &Handle<Font>, label: &str, tag: impl Bundle) {
    p.spawn((
        ButtonBundle {
            style: Style {
                padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::srgb(0.11, 0.17, 0.2).into(),
            ..default()
        },
        tag,
    ))
    .with_children(|b| {
        b.spawn(text_bundle(
            font,
            label,
            18.0,
            Color::srgb(0.91, 0.93, 0.92),
        ));
    });
}

fn build_ui(commands: &mut Commands, font: &Handle<Font>) {
    let panel = Color::srgba(0.07, 0.11, 0.13, 0.88);
    let light = Color::srgb(0.91, 0.93, 0.92);
    // счёт и время
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|p| {
            p.spawn((text_bundle(font, "", 20.0, light), UiTag::Top));
        });
    // лента событий
    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(44.0),
                right: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                row_gap: Val::Px(4.0),
                ..default()
            },
            ..default()
        },
        FeedBox,
    ));
    // панель героя
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(12.0),
                bottom: Val::Px(12.0),
                width: Val::Px(560.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                ..default()
            },
            background_color: panel.into(),
            ..default()
        })
        .with_children(|p| {
            ui_bar(p, UiBar::Hp, Color::srgb(0.35, 0.82, 0.43), 14.0);
            ui_bar(p, UiBar::Mp, Color::srgb(0.31, 0.55, 1.0), 10.0);
            ui_bar(p, UiBar::Xp, Color::srgb(0.75, 0.55, 1.0), 4.0);
            p.spawn((text_bundle(font, "", 15.0, light), UiTag::Hud));
        });

    // стартовое меню
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(14.0),
                    ..default()
                },
                background_color: Color::srgba(0.03, 0.05, 0.06, 0.88).into(),
                ..default()
            },
            MenuRoot,
        ))
        .with_children(|p| {
            p.spawn(text_bundle(font, "ТРОННЫЙ РУБЕЖ", 48.0, Color::srgb(0.89, 0.70, 0.29)));
            let mut about = text_bundle(
                font,
                "Ведите героя и крипов по одной линии. Разрушьте обе башни врага, затем его трон.",
                17.0,
                Color::srgb(0.55, 0.63, 0.65),
            );
            about.style.max_width = Val::Px(640.0);
            p.spawn(about);
            p.spawn(text_bundle(font, "Герой", 16.0, Color::srgb(0.89, 0.70, 0.29)));
            p.spawn(NodeBundle { style: Style { column_gap: Val::Px(10.0), ..default() }, ..default() }).with_children(|r| {
                for (i, h) in HEROES.iter().enumerate() {
                    button(r, font, &format!("{} ({})", h.name, h.role), HeroBtn(i));
                }
            });
            p.spawn(text_bundle(font, "Сложность", 16.0, Color::srgb(0.89, 0.70, 0.29)));
            p.spawn(NodeBundle { style: Style { column_gap: Val::Px(10.0), ..default() }, ..default() }).with_children(|r| {
                for (i, d) in DIFFS.iter().enumerate() {
                    button(r, font, d.name, DiffBtn(i));
                }
            });
            let mut keys = text_bundle(
                font,
                "ПКМ: идти или атаковать (можно зажать)   ЛКМ: бросить способность   Q W E R: способности\nКолесо: приблизить камеру   Край экрана: двигать камеру   Пробел: камера на героя\n1-4: купить   S: стоп   P: пауза   Esc: отмена прицела",
                14.0,
                Color::srgb(0.55, 0.63, 0.65),
            );
            keys.style.max_width = Val::Px(760.0);
            p.spawn(keys);
            button(p, font, "Начать бой (Enter)", StartBtn);
        });

    // экран конца боя
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(16.0),
                    ..default()
                },
                background_color: Color::srgba(0.03, 0.05, 0.06, 0.8).into(),
                ..default()
            },
            EndRoot,
        ))
        .with_children(|p| {
            p.spawn((
                text_bundle(font, "", 56.0, Color::srgb(0.89, 0.70, 0.29)),
                UiTag::EndTitle,
            ));
            p.spawn((text_bundle(font, "", 20.0, light), UiTag::EndStats));
            button(p, font, "Сыграть ещё", AgainBtn);
        });
}

// ---------- меню ----------
fn start_game(s: &mut Session, commands: &mut Commands, views: &mut Views) {
    for (_, e) in views
        .units
        .drain()
        .chain(views.projs.drain())
        .chain(views.zones.drain())
        .chain(views.bars.drain())
    {
        commands.entity(e).despawn_recursive();
    }
    for e in views.ruins.drain(..) {
        commands.entity(e).despawn_recursive();
    }
    s.game = Game::new(s.hero, s.diff, seed_now());
    s.screen = Screen::Playing;
    s.aim = None;
    s.locked = true;
    let p = s.game.player();
    s.focus = Vec2::new(s.game.units[p].x, s.game.units[p].y);
    s.game.events.push(Ev::Feed {
        msg: "Разрушьте обе башни, затем трон врага".to_string(),
        team: 0,
    });
}

#[allow(clippy::too_many_arguments)]
fn menu_system(
    mut commands: Commands,
    mut sess: ResMut<Session>,
    mut views: ResMut<Views>,
    keys: Res<ButtonInput<KeyCode>>,
    pressed: Query<
        (
            &Interaction,
            Option<&HeroBtn>,
            Option<&DiffBtn>,
            Option<&StartBtn>,
            Option<&AgainBtn>,
        ),
        Changed<Interaction>,
    >,
    mut colors: Query<(&mut BackgroundColor, Option<&HeroBtn>, Option<&DiffBtn>)>,
    mut menu: Query<&mut Style, With<MenuRoot>>,
    mut end: Query<&mut Style, (With<EndRoot>, Without<MenuRoot>)>,
) {
    let s = &mut *sess;
    let mut start = false;
    let mut again = false;
    for (i, h, d, st, ag) in pressed.iter() {
        if *i != Interaction::Pressed {
            continue;
        }
        if let Some(h) = h {
            s.hero = h.0;
        }
        if let Some(d) = d {
            s.diff = d.0;
        }
        if st.is_some() {
            start = true;
        }
        if ag.is_some() {
            again = true;
        }
    }
    if s.screen == Screen::Menu && keys.just_pressed(KeyCode::Enter) {
        start = true;
    }
    if s.screen == Screen::Over && keys.just_pressed(KeyCode::Enter) {
        again = true;
    }
    if start && s.screen == Screen::Menu {
        start_game(s, &mut commands, &mut views);
        for mut st in menu.iter_mut() {
            st.display = Display::None;
        }
    }
    if again && s.screen == Screen::Over {
        s.screen = Screen::Menu;
        for mut st in end.iter_mut() {
            st.display = Display::None;
        }
        for mut st in menu.iter_mut() {
            st.display = Display::Flex;
        }
    }
    for (mut bg, h, d) in colors.iter_mut() {
        let on = h.map_or(false, |h| h.0 == s.hero) || d.map_or(false, |d| d.0 == s.diff);
        let tagged = h.is_some() || d.is_some();
        if tagged {
            *bg = if on {
                Color::srgb(0.36, 0.28, 0.1).into()
            } else {
                Color::srgb(0.11, 0.17, 0.2).into()
            };
        }
    }
}

// ---------- ввод ----------
fn ground_at(cam: &Camera, gt: &GlobalTransform, p: Vec2) -> Option<Vec2> {
    let ray = cam.viewport_to_world(gt, p)?;
    let d: Vec3 = *ray.direction;
    if d.y.abs() < 1e-5 {
        return None;
    }
    let t = -ray.origin.y / d.y;
    if t < 0.0 {
        return None;
    }
    let p3 = ray.origin + d * t;
    Some(Vec2::new(p3.x / S, p3.z / S))
}

fn say(s: &mut Session, x: f32, y: f32, msg: &str, c: [f32; 3]) {
    s.game.events.push(Ev::Text {
        x,
        y,
        s: msg.to_string(),
        col: c,
    });
}

fn issue_order(s: &mut Session, quiet: bool) {
    let p = s.game.player();
    if s.game.units[p].dead || s.game.over {
        return;
    }
    let target = s
        .hover
        .and_then(|id| s.game.idx(id))
        .filter(|&j| !s.game.invuln(j));
    if let Some(j) = target {
        let (id, x, y, r) = (
            s.game.units[j].id,
            s.game.units[j].x,
            s.game.units[j].y,
            s.game.units[j].r,
        );
        s.game.units[p].order = Order::Attack(id);
        if !quiet {
            s.game.events.push(Ev::Ring {
                x,
                y,
                r0: r + 16.0,
                r1: r + 4.0,
                dur: 0.3,
                col: [1.0, 0.42, 0.42],
            });
        }
    } else {
        let x = s.mouse.x.clamp(20.0, W - 20.0);
        let y = s.mouse.y.clamp(20.0, H - 20.0);
        s.game.units[p].order = Order::Move(x, y);
        if !quiet {
            s.game.events.push(Ev::Ring {
                x,
                y,
                r0: 26.0,
                r1: 6.0,
                dur: 0.35,
                col: [0.62, 0.89, 0.69],
            });
        }
    }
}

fn press_ability(s: &mut Session, i: usize) {
    let p = s.game.player();
    if s.game.units[p].dead || s.game.units[p].stun > 0.0 {
        return;
    }
    let (px, py) = (s.game.units[p].x, s.game.units[p].y);
    let hd = s.game.hd(p);
    let a = HEROES[hd.key].abilities[i];
    if hd.cds[i] > 0.0 {
        say(s, px, py - 50.0, "Перезарядка", [0.62, 0.7, 0.72]);
        return;
    }
    if hd.mp < a.mana {
        say(s, px, py - 50.0, "Мало маны", [0.5, 0.69, 1.0]);
        return;
    }
    if a.ty == AbType::SelfAoe || a.ty == AbType::Buff {
        s.aim = None;
        s.game.cast(p, i, px, py);
    } else if s.aim == Some(i) {
        s.aim = None;
        let m = s.mouse;
        s.game.cast(p, i, m.x, m.y);
    } else {
        s.aim = Some(i);
    }
}

#[allow(clippy::too_many_arguments)]
fn input_system(
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cams: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut wheel: EventReader<MouseWheel>,
    mut sess: ResMut<Session>,
) {
    let s = &mut *sess;
    let dt = time.delta_seconds();
    if keys.just_pressed(KeyCode::KeyP) {
        s.screen = match s.screen {
            Screen::Playing => Screen::Paused,
            Screen::Paused => Screen::Playing,
            other => other,
        };
    }
    if s.screen != Screen::Playing {
        wheel.clear();
        return;
    }
    let (Ok(w), Ok((cam, gt))) = (windows.get_single(), cams.get_single()) else {
        return;
    };
    let cursor = w.cursor_position();

    if let Some(c) = cursor {
        if let Some(g) = ground_at(cam, gt, c) {
            s.mouse = g;
        }
    }
    // что под курсором
    s.hover = None;
    if let Some(c) = cursor {
        let mut best = f32::MAX;
        for u in s.game.units.iter().filter(|u| u.team == 1 && !u.dead) {
            let h = if u.is_struct() {
                50.0
            } else if u.kind == Kind::Hero {
                40.0
            } else {
                15.0
            };
            let p3 = Vec3::new(u.x * S, h * S, u.y * S);
            if let (Some(a), Some(b)) = (
                cam.world_to_viewport(gt, p3),
                cam.world_to_viewport(gt, p3 + Vec3::X * (u.r + 10.0) * S),
            ) {
                let rad = (b - a).length() + 8.0;
                let d = (a - c).length();
                if d <= rad && d < best {
                    best = d;
                    s.hover = Some(u.id);
                }
            }
        }
    }

    // мышь
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(i) = s.aim.take() {
            let p = s.game.player();
            let m = s.mouse;
            s.game.cast(p, i, m.x, m.y);
        } else {
            issue_order(s, false);
        }
    }
    if mouse.just_pressed(MouseButton::Right) {
        if s.aim.is_some() {
            s.aim = None;
        } else {
            issue_order(s, false);
            s.hold_t = 0.25;
        }
    }
    if mouse.pressed(MouseButton::Right) && s.aim.is_none() {
        s.hold_t -= dt;
        if s.hold_t <= 0.0 {
            s.hold_t = 0.12;
            issue_order(s, true);
        }
    }

    // клавиши
    for (k, i) in [
        (KeyCode::KeyQ, 0usize),
        (KeyCode::KeyW, 1),
        (KeyCode::KeyE, 2),
        (KeyCode::KeyR, 3),
    ] {
        if keys.just_pressed(k) {
            press_ability(s, i);
        }
    }
    for (k, i) in [
        (KeyCode::Digit1, 0usize),
        (KeyCode::Digit2, 1),
        (KeyCode::Digit3, 2),
        (KeyCode::Digit4, 3),
    ] {
        if keys.just_pressed(k) {
            let p = s.game.player();
            if !s.game.buy(p, i) {
                let (x, y) = (s.game.units[p].x, s.game.units[p].y);
                say(s, x, y - 50.0, "Не хватает золота", [0.89, 0.70, 0.29]);
            }
        }
    }
    if keys.just_pressed(KeyCode::Escape) {
        s.aim = None;
    }
    if keys.just_pressed(KeyCode::KeyS) {
        let p = s.game.player();
        s.game.units[p].order = Order::None;
    }
    if keys.just_pressed(KeyCode::Space) {
        s.locked = true;
    }

    // камера: колесо и край экрана
    for ev in wheel.read() {
        s.dist = (s.dist * (1.0 - ev.y * 0.08)).clamp(45.0, 150.0);
    }
    let mut dx = 0.0;
    let mut dy = 0.0;
    if let Some(c) = cursor {
        let m = 20.0;
        if c.x < m {
            dx = -1.0;
        } else if c.x > w.width() - m {
            dx = 1.0;
        }
        if c.y < m {
            dy = -1.0;
        } else if c.y > w.height() - m {
            dy = 1.0;
        }
    }
    if dx == 0.0 && dy == 0.0 {
        s.edge_t = 0.0;
    } else {
        s.edge_t += dt;
        if s.edge_t > 0.35 {
            s.locked = false;
            let sp = 1000.0 * dt * (s.dist / 82.0);
            s.focus.x = (s.focus.x + dx * sp).clamp(0.0, W);
            s.focus.y = (s.focus.y + dy * sp).clamp(0.0, H);
        }
    }
}

fn step_game(time: Res<Time>, mut sess: ResMut<Session>) {
    let s = &mut *sess;
    if s.screen == Screen::Playing {
        let dt = time.delta_seconds().min(0.05);
        s.game.update(dt);
        if s.game.over {
            s.screen = Screen::Over;
        }
    }
}

// ---------- события из логики ----------
#[allow(clippy::too_many_arguments)]
fn consume_events(
    mut commands: Commands,
    mut sess: ResMut<Session>,
    font: Res<UiFont>,
    gfx: Res<Gfx>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    feed_box: Query<Entity, With<FeedBox>>,
) {
    let s = &mut *sess;
    let evs = std::mem::take(&mut s.game.events);
    for ev in evs {
        match ev {
            Ev::Ring {
                x,
                y,
                r0,
                r1,
                dur,
                col: c,
            } => {
                commands.spawn((
                    PbrBundle {
                        mesh: gfx.cyl.clone(),
                        material: ghost(&mut mats, c, 0.8),
                        transform: Transform {
                            translation: Vec3::new(x * S, 0.3, y * S),
                            scale: Vec3::new(r0 * S, 0.06, r0 * S),
                            ..default()
                        },
                        ..default()
                    },
                    Fade {
                        t: 0.0,
                        dur,
                        r0,
                        r1,
                        rgb: c,
                    },
                ));
            }
            Ev::Text {
                x,
                y,
                s: msg,
                col: c,
            } => {
                commands.spawn((
                    TextBundle::from_section(
                        msg,
                        TextStyle {
                            font: font.0.clone(),
                            font_size: 20.0,
                            color: col(c),
                        },
                    )
                    .with_style(Style {
                        position_type: PositionType::Absolute,
                        left: Val::Px(-200.0),
                        top: Val::Px(-200.0),
                        ..default()
                    }),
                    FloatText {
                        pos: Vec3::new(x * S, 3.8, y * S),
                        t: 0.0,
                        rgb: c,
                    },
                ));
            }
            Ev::Feed { msg, team } => {
                if let Ok(b) = feed_box.get_single() {
                    commands.entity(b).with_children(|p| {
                        p.spawn((
                            NodeBundle {
                                style: Style {
                                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                                    border: UiRect::right(Val::Px(3.0)),
                                    ..default()
                                },
                                background_color: Color::srgba(0.04, 0.06, 0.07, 0.8).into(),
                                border_color: col(TEAM_COL[team]).into(),
                                ..default()
                            },
                            FeedItem { ttl: 5.5 },
                        ))
                        .with_children(|q| {
                            q.spawn(text_bundle(
                                &font.0,
                                &msg,
                                15.0,
                                Color::srgb(0.91, 0.93, 0.92),
                            ));
                        });
                    });
                }
            }
            Ev::End { .. } => {}
        }
    }
}

// ---------- синхронизация модели и сцены ----------
fn spawn_model(
    commands: &mut Commands,
    gfx: &Gfx,
    mats: &mut Assets<StandardMaterial>,
    u: &Unit,
) -> Entity {
    let tc = TEAM_COL[u.team];
    let team_m = solid(mats, tc);
    let dark_m = solid(mats, [tc[0] * 0.45, tc[1] * 0.45, tc[2] * 0.45]);
    let light_m = glowm(
        mats,
        [tc[0] * 0.5 + 0.5, tc[1] * 0.5 + 0.5, tc[2] * 0.5 + 0.5],
    );
    let ring_m = ghost(mats, tc, 0.9);
    let skin = solid(mats, hex(0xe3c7a5));
    let metal = solid(mats, hex(0xbfc9ce));
    let wood = solid(mats, hex(0x7a5a38));
    let stone = solid(mats, hex(0x6d7a84));
    let (cube, sphere, cyl) = (gfx.cube.clone(), gfx.sphere.clone(), gfx.cyl.clone());
    let r = u.r;
    let mut ent = commands.spawn((
        SpatialBundle {
            transform: Transform {
                translation: wp(u.x, u.y),
                rotation: Quat::from_rotation_y(-u.face),
                scale: Vec3::splat(S),
            },
            ..default()
        },
        UnitView,
    ));
    let id = u.id;
    let kind = u.kind;
    let ranged = u.ranged;
    let key = u.hero.as_ref().map(|h| h.key).unwrap_or(0);
    ent.with_children(|p| {
        if kind == Kind::Creep {
            part(
                p,
                &cyl,
                &ring_m,
                Vec3::new(r * 1.7, 0.6, r * 1.7),
                Vec3::new(0.0, 1.0, 0.0),
            );
            part(
                p,
                &cyl,
                &team_m,
                Vec3::new(r * 0.72, r * 1.5, r * 0.72),
                Vec3::new(0.0, r * 0.75 + 2.0, 0.0),
            );
            part(
                p,
                &sphere,
                &skin,
                Vec3::splat(r * 0.5),
                Vec3::new(0.0, r * 1.85, 0.0),
            );
            part(
                p,
                &cyl,
                &dark_m,
                Vec3::new(r * 0.45, r * 0.5, r * 0.45),
                Vec3::new(0.0, r * 2.4, 0.0),
            );
            if ranged {
                part(
                    p,
                    &cyl,
                    &wood,
                    Vec3::new(1.5, r * 2.4, 1.5),
                    Vec3::new(r * 0.5, r * 1.7, r * 0.75),
                );
                part(
                    p,
                    &sphere,
                    &light_m,
                    Vec3::splat(r * 0.28),
                    Vec3::new(r * 0.5, r * 2.9, r * 0.75),
                );
            } else {
                part(
                    p,
                    &cube,
                    &metal,
                    Vec3::new(r * 1.7, 3.0, 3.0),
                    Vec3::new(r * 1.4, r * 1.1, r * 0.75),
                );
            }
        }
    });
    let eid = ent.id();
    match kind {
        Kind::Hero => {
            let hc = HEROES[key].color;
            let body = solid(mats, hc);
            let dark = solid(mats, hex(0x39424b));
            let purple = solid(mats, hex(0x3d2a66));
            let green = solid(mats, hex(0x2f5a3c));
            let bone = solid(mats, hex(0xeeeeee));
            let orb = glowm(mats, hex(0xe5d0ff));
            commands.entity(eid).with_children(|p| {
                part(
                    p,
                    &cyl,
                    &ring_m,
                    Vec3::new(r * 1.9, 0.6, r * 1.9),
                    Vec3::new(0.0, 1.2, 0.0),
                );
                match key {
                    0 => {
                        part(
                            p,
                            &cyl,
                            &dark,
                            Vec3::new(6.0, 22.0, 6.0),
                            Vec3::new(0.0, 11.0, -7.0),
                        );
                        part(
                            p,
                            &cyl,
                            &dark,
                            Vec3::new(6.0, 22.0, 6.0),
                            Vec3::new(0.0, 11.0, 7.0),
                        );
                        part(
                            p,
                            &cyl,
                            &body,
                            Vec3::new(13.0, 28.0, 13.0),
                            Vec3::new(0.0, 36.0, 0.0),
                        );
                        part(
                            p,
                            &sphere,
                            &skin,
                            Vec3::splat(10.0),
                            Vec3::new(0.0, 61.0, 0.0),
                        );
                        part(
                            p,
                            &cyl,
                            &metal,
                            Vec3::new(11.5, 15.0, 11.5),
                            Vec3::new(0.0, 72.0, 0.0),
                        );
                        part(
                            p,
                            &sphere,
                            &metal,
                            Vec3::splat(8.0),
                            Vec3::new(0.0, 49.0, -16.0),
                        );
                        part(
                            p,
                            &sphere,
                            &metal,
                            Vec3::splat(8.0),
                            Vec3::new(0.0, 49.0, 16.0),
                        );
                        part(
                            p,
                            &cube,
                            &stone,
                            Vec3::new(4.0, 26.0, 18.0),
                            Vec3::new(6.0, 36.0, -20.0),
                        );
                        part(
                            p,
                            &cube,
                            &metal,
                            Vec3::new(36.0, 3.0, 6.0),
                            Vec3::new(29.0, 38.0, 18.0),
                        );
                    }
                    1 => {
                        part(
                            p,
                            &cyl,
                            &body,
                            Vec3::new(17.0, 50.0, 17.0),
                            Vec3::new(0.0, 25.0, 0.0),
                        );
                        part(
                            p,
                            &sphere,
                            &skin,
                            Vec3::splat(9.0),
                            Vec3::new(0.0, 58.0, 0.0),
                        );
                        part(
                            p,
                            &cyl,
                            &purple,
                            Vec3::new(12.0, 24.0, 12.0),
                            Vec3::new(0.0, 73.0, 0.0),
                        );
                        part(
                            p,
                            &cyl,
                            &purple,
                            Vec3::new(15.0, 3.0, 15.0),
                            Vec3::new(0.0, 62.0, 0.0),
                        );
                        part(
                            p,
                            &cyl,
                            &wood,
                            Vec3::new(1.7, 74.0, 1.7),
                            Vec3::new(6.0, 37.0, 17.0),
                        );
                        part(
                            p,
                            &sphere,
                            &orb,
                            Vec3::splat(6.0),
                            Vec3::new(6.0, 76.0, 17.0),
                        );
                    }
                    _ => {
                        part(
                            p,
                            &cyl,
                            &dark,
                            Vec3::new(5.0, 22.0, 5.0),
                            Vec3::new(0.0, 11.0, -6.0),
                        );
                        part(
                            p,
                            &cyl,
                            &dark,
                            Vec3::new(5.0, 22.0, 5.0),
                            Vec3::new(0.0, 11.0, 6.0),
                        );
                        part(
                            p,
                            &cyl,
                            &body,
                            Vec3::new(11.0, 26.0, 11.0),
                            Vec3::new(0.0, 34.0, 0.0),
                        );
                        part(
                            p,
                            &sphere,
                            &skin,
                            Vec3::splat(9.0),
                            Vec3::new(0.0, 55.0, 0.0),
                        );
                        part(
                            p,
                            &cyl,
                            &green,
                            Vec3::new(11.0, 15.0, 11.0),
                            Vec3::new(0.0, 66.0, 0.0),
                        );
                        part(
                            p,
                            &cube,
                            &wood,
                            Vec3::new(4.0, 22.0, 6.0),
                            Vec3::new(-9.0, 41.0, -5.0),
                        );
                        part(
                            p,
                            &cyl,
                            &wood,
                            Vec3::new(1.2, 32.0, 1.2),
                            Vec3::new(8.0, 40.0, 16.0),
                        );
                        part(
                            p,
                            &cube,
                            &bone,
                            Vec3::new(30.0, 0.8, 0.8),
                            Vec3::new(20.0, 40.0, 16.0),
                        );
                    }
                }
            });
        }
        Kind::Tower => {
            let shield_m = ghost(mats, [1.0, 1.0, 1.0], 0.13);
            let mut shield_e: Option<Entity> = None;
            commands.entity(eid).with_children(|p| {
                part(
                    p,
                    &cyl,
                    &stone,
                    Vec3::new(46.0, 16.0, 46.0),
                    Vec3::new(0.0, 8.0, 0.0),
                );
                part(
                    p,
                    &cyl,
                    &dark_m,
                    Vec3::new(27.0, 88.0, 27.0),
                    Vec3::new(0.0, 60.0, 0.0),
                );
                part(
                    p,
                    &cyl,
                    &team_m,
                    Vec3::new(28.5, 6.0, 28.5),
                    Vec3::new(0.0, 44.0, 0.0),
                );
                part(
                    p,
                    &cyl,
                    &stone,
                    Vec3::new(34.0, 10.0, 34.0),
                    Vec3::new(0.0, 109.0, 0.0),
                );
                part(
                    p,
                    &sphere,
                    &team_m,
                    Vec3::new(30.0, 30.0, 30.0),
                    Vec3::new(0.0, 122.0, 0.0),
                );
                part(
                    p,
                    &sphere,
                    &light_m,
                    Vec3::splat(11.0),
                    Vec3::new(0.0, 156.0, 0.0),
                );
                shield_e = Some(part(
                    p,
                    &sphere,
                    &shield_m,
                    Vec3::splat(70.0),
                    Vec3::new(0.0, 85.0, 0.0),
                ));
            });
            if let Some(sh) = shield_e {
                commands.entity(sh).insert(Shield(id));
            }
        }
        Kind::Throne => {
            let shield_m = ghost(mats, [1.0, 1.0, 1.0], 0.12);
            let mut shield_e: Option<Entity> = None;
            let mut crystal_e: Option<Entity> = None;
            commands.entity(eid).with_children(|p| {
                part(
                    p,
                    &cyl,
                    &stone,
                    Vec3::new(80.0, 14.0, 80.0),
                    Vec3::new(0.0, 7.0, 0.0),
                );
                part(
                    p,
                    &cyl,
                    &dark_m,
                    Vec3::new(58.0, 10.0, 58.0),
                    Vec3::new(0.0, 19.0, 0.0),
                );
                for (x, z) in [(1.0f32, 1.0f32), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
                    part(
                        p,
                        &cube,
                        &stone,
                        Vec3::new(10.0, 50.0, 10.0),
                        Vec3::new(x * 52.0, 42.0, z * 52.0),
                    );
                    part(
                        p,
                        &sphere,
                        &team_m,
                        Vec3::splat(8.0),
                        Vec3::new(x * 52.0, 72.0, z * 52.0),
                    );
                }
                crystal_e = Some(part(
                    p,
                    &sphere,
                    &light_m,
                    Vec3::new(26.0, 44.0, 26.0),
                    Vec3::new(0.0, 98.0, 0.0),
                ));
                shield_e = Some(part(
                    p,
                    &sphere,
                    &shield_m,
                    Vec3::splat(95.0),
                    Vec3::new(0.0, 70.0, 0.0),
                ));
            });
            if let Some(c) = crystal_e {
                commands.entity(c).insert(Spin);
            }
            if let Some(sh) = shield_e {
                commands.entity(sh).insert(Shield(id));
            }
        }
        Kind::Creep => {}
    }
    eid
}

fn spawn_bar(commands: &mut Commands, id: u32, w: f32, h: f32, up: f32, color: Color) -> Entity {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(-100.0),
                    top: Val::Px(-100.0),
                    width: Val::Px(w),
                    height: Val::Px(h),
                    ..default()
                },
                background_color: Color::srgba(0.0, 0.0, 0.0, 0.7).into(),
                ..default()
            },
            HpBar { id, w, up },
        ))
        .with_children(|p| {
            p.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: color.into(),
                    ..default()
                },
                HpFill,
            ));
        })
        .id()
}

#[allow(clippy::too_many_arguments)]
fn sync_views(
    mut commands: Commands,
    sess: Res<Session>,
    mut views: ResMut<Views>,
    gfx: Res<Gfx>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut unit_q: Query<
        (&mut Transform, &mut Visibility),
        (With<UnitView>, Without<ProjView>, Without<ZoneView>),
    >,
    mut proj_q: Query<&mut Transform, (With<ProjView>, Without<UnitView>, Without<ZoneView>)>,
    mut zone_q: Query<&mut Transform, (With<ZoneView>, Without<UnitView>, Without<ProjView>)>,
    mut shields: Query<
        (&Shield, &mut Visibility),
        (Without<UnitView>, Without<ProjView>, Without<ZoneView>),
    >,
) {
    let s = &*sess;
    let g = &s.game;
    let views = &mut *views;

    // юниты
    let mut alive: Vec<u32> = Vec::with_capacity(g.units.len());
    for u in g.units.iter() {
        if u.dead && u.kind != Kind::Hero {
            continue;
        }
        alive.push(u.id);
        let e = match views.units.get(&u.id) {
            Some(e) => *e,
            None => {
                let e = spawn_model(&mut commands, &gfx, &mut mats, u);
                views.units.insert(u.id, e);
                let bar_color = if u.team == 0 {
                    Color::srgb(0.37, 0.88, 0.48)
                } else {
                    Color::srgb(1.0, 0.39, 0.39)
                };
                let (w, up, color) = match u.kind {
                    Kind::Hero => (64.0, 11.0, bar_color),
                    Kind::Creep => (28.0, 5.4, bar_color),
                    Kind::Tower => (84.0, 19.0, bar_color),
                    Kind::Throne => (120.0, 16.0, bar_color),
                };
                let h = if u.kind == Kind::Hero {
                    9.0
                } else if u.kind == Kind::Creep {
                    4.0
                } else {
                    7.0
                };
                let b = spawn_bar(&mut commands, u.id, w, h, up, color);
                views.bars.insert(u.id, b);
                e
            }
        };
        if let Ok((mut t, mut v)) = unit_q.get_mut(e) {
            let mut lift = 0.0;
            if let Some(h) = &u.hero {
                if let Some(d) = &h.dash {
                    lift = ((d.t / d.dur).min(1.0) * std::f32::consts::PI).sin() * 22.0 * S;
                }
            }
            t.translation = wp(u.x, u.y) + Vec3::Y * lift;
            t.rotation = Quat::from_rotation_y(-u.face);
            *v = if u.dead {
                Visibility::Hidden
            } else {
                Visibility::Visible
            };
        }
    }
    views.units.retain(|id, e| {
        if alive.contains(id) {
            true
        } else {
            commands.entity(*e).despawn_recursive();
            false
        }
    });
    views.bars.retain(|id, e| {
        if alive.contains(id) {
            true
        } else {
            commands.entity(*e).despawn_recursive();
            false
        }
    });

    // щиты неуязвимых построек
    for (sh, mut v) in shields.iter_mut() {
        let on = g.idx(sh.0).map_or(false, |i| g.invuln(i));
        *v = if on {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // снаряды
    let mut palive: Vec<u32> = Vec::new();
    for p in g.projs.iter() {
        palive.push(p.id);
        let e = match views.projs.get(&p.id) {
            Some(e) => *e,
            None => {
                let m = glowm(&mut mats, if p.homing { p.col } else { [1.0, 1.0, 1.0] });
                let (mesh, scale) = if p.homing {
                    (gfx.sphere.clone(), Vec3::splat(0.5))
                } else {
                    (gfx.cube.clone(), Vec3::new(5.4, 0.5, 0.5))
                };
                let e = commands
                    .spawn((
                        PbrBundle {
                            mesh,
                            material: m,
                            transform: Transform {
                                translation: Vec3::new(p.x * S, 3.2, p.y * S),
                                rotation: Quat::from_rotation_y(-p.dy.atan2(p.dx)),
                                scale,
                            },
                            ..default()
                        },
                        ProjView,
                    ))
                    .id();
                views.projs.insert(p.id, e);
                e
            }
        };
        if let Ok(mut t) = proj_q.get_mut(e) {
            t.translation = Vec3::new(p.x * S, 3.2, p.y * S);
        }
    }
    views.projs.retain(|id, e| {
        if palive.contains(id) {
            true
        } else {
            commands.entity(*e).despawn_recursive();
            false
        }
    });

    // зоны (метеор, буря)
    let mut zalive: Vec<u32> = Vec::new();
    for z in g.zones.iter() {
        zalive.push(z.id);
        let e = match views.zones.get(&z.id) {
            Some(e) => *e,
            None => {
                let e = commands
                    .spawn((
                        PbrBundle {
                            mesh: gfx.cyl.clone(),
                            material: ghost(&mut mats, z.col, 0.3),
                            transform: Transform {
                                translation: Vec3::new(z.x * S, 0.25, z.y * S),
                                scale: Vec3::new(0.01, 0.05, 0.01),
                                ..default()
                            },
                            ..default()
                        },
                        ZoneView,
                    ))
                    .id();
                views.zones.insert(z.id, e);
                e
            }
        };
        if let Ok(mut t) = zone_q.get_mut(e) {
            let k = (z.t / z.delay).clamp(0.02, 1.0);
            t.scale = Vec3::new(z.r * S * k, 0.05, z.r * S * k);
        }
    }
    views.zones.retain(|id, e| {
        if zalive.contains(id) {
            true
        } else {
            commands.entity(*e).despawn_recursive();
            false
        }
    });

    // руины на месте башен
    while views.ruins.len() < g.ruins.len() {
        let (rx, ry) = g.ruins[views.ruins.len()];
        let m = solid(&mut mats, hex(0x3d342c));
        let e = commands
            .spawn(SpatialBundle {
                transform: Transform::from_translation(wp(rx, ry)),
                ..default()
            })
            .with_children(|p| {
                for i in 0..6 {
                    let a = i as f32 * 1.05;
                    part(
                        p,
                        &gfx.cube,
                        &m,
                        Vec3::new(1.2 + (i % 3) as f32 * 0.5, 0.8, 1.4),
                        Vec3::new(a.cos() * 2.4, 0.4, a.sin() * 2.4),
                    );
                }
            })
            .id();
        views.ruins.push(e);
    }
}

fn camera_system(
    time: Res<Time>,
    mut sess: ResMut<Session>,
    mut q: Query<&mut Transform, With<MainCam>>,
) {
    let s = &mut *sess;
    let dt = time.delta_seconds();
    if s.locked {
        let p = s.game.player();
        let target = Vec2::new(s.game.units[p].x, s.game.units[p].y);
        let k = (dt * 10.0).min(1.0);
        let f = s.focus;
        s.focus = f + (target - f) * k;
    }
    s.focus.x = s.focus.x.clamp(0.0, W);
    s.focus.y = s.focus.y.clamp(0.0, H);
    let target = Vec3::new(s.focus.x * S, 0.0, s.focus.y * S);
    let pos = target + Vec3::new(0.0, PITCH.sin() * s.dist, PITCH.cos() * s.dist);
    if let Ok(mut t) = q.get_single_mut() {
        *t = Transform::from_translation(pos).looking_at(target, Vec3::Y);
    }
}

fn update_bars(
    sess: Res<Session>,
    cams: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    mut bars: Query<(&HpBar, &mut Style, &mut Visibility, &Children), Without<HpFill>>,
    mut fills: Query<&mut Style, (With<HpFill>, Without<HpBar>)>,
) {
    let Ok((cam, gt)) = cams.get_single() else {
        return;
    };
    let g = &sess.game;
    for (bar, mut st, mut vis, children) in bars.iter_mut() {
        let u = g.idx(bar.id).map(|i| &g.units[i]).filter(|u| !u.dead);
        let Some(u) = u else {
            *vis = Visibility::Hidden;
            continue;
        };
        match cam.world_to_viewport(gt, Vec3::new(u.x * S, bar.up, u.y * S)) {
            Some(p) => {
                *vis = Visibility::Visible;
                st.left = Val::Px(p.x - bar.w / 2.0);
                st.top = Val::Px(p.y);
                for c in children.iter() {
                    if let Ok(mut f) = fills.get_mut(*c) {
                        f.width = Val::Percent((u.hp / u.max_hp * 100.0).clamp(0.0, 100.0));
                    }
                }
            }
            None => *vis = Visibility::Hidden,
        }
    }
}

fn update_hud(
    sess: Res<Session>,
    mut texts: Query<(&UiTag, &mut Text)>,
    mut bars: Query<(&UiBar, &mut Style)>,
) {
    let s = &*sess;
    let g = &s.game;
    let p = g.player();
    let u = &g.units[p];
    let hd = g.hd(p);
    for (tag, mut t) in texts.iter_mut() {
        match tag {
            UiTag::Top => {
                let tt = g.t as u32;
                let state = if s.screen == Screen::Paused {
                    "   ПАУЗА"
                } else {
                    ""
                };
                t.sections[0].value = format!(
                    "{} {}  :  {} {}     {:02}:{:02}     Волна {}{}",
                    TEAM_NAME[0],
                    g.kills[0],
                    g.kills[1],
                    TEAM_NAME[1],
                    tt / 60,
                    tt % 60,
                    g.wave,
                    state
                );
            }
            UiTag::Hud => {
                let def = &HEROES[hd.key];
                let mut o = String::new();
                o.push_str(&format!(
                    "{}  ур. {}     Золото {}     {}\n",
                    def.name,
                    hd.level,
                    hd.gold as i32,
                    if u.dead {
                        format!("Возрождение: {} с", hd.respawn.ceil() as i32)
                    } else {
                        format!(
                            "Здоровье {}/{}  Мана {}/{}",
                            u.hp.ceil() as i32,
                            u.max_hp as i32,
                            hd.mp.floor() as i32,
                            hd.max_mp as i32
                        )
                    }
                ));
                for i in 0..4 {
                    let a = def.abilities[i];
                    let status = if hd.cds[i] > 0.0 {
                        format!("{} с", hd.cds[i].ceil() as i32)
                    } else if hd.mp < a.mana {
                        "мало маны".to_string()
                    } else {
                        "готово".to_string()
                    };
                    let mark = if s.aim == Some(i) { ">" } else { " " };
                    o.push_str(&format!(
                        "{}[{}] {} ({} маны)  {}\n",
                        mark, a.key, a.name, a.mana as i32, status
                    ));
                }
                for i in 0..4 {
                    o.push_str(&format!(
                        "[{}] {}: {}\n",
                        i + 1,
                        ITEM_NAMES[i],
                        g.item_cost(p, i) as i32
                    ));
                }
                o.push_str(&format!(
                    "Урон {}  Скорость {}  Дальность {}",
                    u.dmg.round() as i32,
                    u.speed.round() as i32,
                    u.range.round() as i32
                ));
                t.sections[0].value = o;
            }
            _ => {}
        }
    }
    for (b, mut st) in bars.iter_mut() {
        let k = match b {
            UiBar::Hp => u.hp / u.max_hp,
            UiBar::Mp => hd.mp / hd.max_mp,
            UiBar::Xp => {
                if hd.level >= 12 {
                    1.0
                } else {
                    hd.xp / xp_need(hd.level)
                }
            }
        };
        st.width = Val::Percent((k * 100.0).clamp(0.0, 100.0));
    }
}

fn end_screen(
    sess: Res<Session>,
    mut end: Query<&mut Style, With<EndRoot>>,
    mut texts: Query<(&UiTag, &mut Text)>,
) {
    if sess.screen != Screen::Over {
        return;
    }
    for mut st in end.iter_mut() {
        if st.display == Display::None {
            st.display = Display::Flex;
            let g = &sess.game;
            let p = g.player();
            let win = g.winner == 0;
            for (tag, mut t) in texts.iter_mut() {
                match tag {
                    UiTag::EndTitle => {
                        t.sections[0].value = if win {
                            "ПОБЕДА".to_string()
                        } else {
                            "ПОРАЖЕНИЕ".to_string()
                        }
                    }
                    UiTag::EndStats => {
                        let tt = g.t as u32;
                        t.sections[0].value = format!(
                            "{}\nВремя боя {:02}:{:02}   Убийства героев {} : {}   Уровень героя {}",
                            if win { "Трон Багрового ордена разрушен." } else { "Ваш трон разрушен." },
                            tt / 60,
                            tt % 60,
                            g.kills[0],
                            g.kills[1],
                            g.hd(p).level
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}

// ---------- эффекты и мелочи ----------
fn fade_system(
    mut commands: Commands,
    time: Res<Time>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut q: Query<(Entity, &mut Fade, &mut Transform, &Handle<StandardMaterial>)>,
) {
    let dt = time.delta_seconds();
    for (e, mut f, mut t, h) in q.iter_mut() {
        f.t += dt;
        let k = f.t / f.dur;
        if k >= 1.0 {
            commands.entity(e).despawn_recursive();
            continue;
        }
        let r = f.r0 + (f.r1 - f.r0) * (1.0 - (1.0 - k) * (1.0 - k));
        t.scale = Vec3::new(r * S, 0.06, r * S);
        if let Some(m) = mats.get_mut(h) {
            m.base_color = Color::srgba(f.rgb[0], f.rgb[1], f.rgb[2], (1.0 - k) * 0.75);
        }
    }
}

fn float_text_system(
    mut commands: Commands,
    time: Res<Time>,
    cams: Query<(&Camera, &GlobalTransform), With<MainCam>>,
    mut q: Query<(Entity, &mut FloatText, &mut Style, &mut Text)>,
) {
    let dt = time.delta_seconds();
    let Ok((cam, gt)) = cams.get_single() else {
        return;
    };
    for (e, mut f, mut st, mut tx) in q.iter_mut() {
        f.t += dt;
        if f.t > 0.9 {
            commands.entity(e).despawn_recursive();
            continue;
        }
        let pos = f.pos + Vec3::Y * f.t * 4.0;
        match cam.world_to_viewport(gt, pos) {
            Some(p) => {
                st.left = Val::Px(p.x - 24.0);
                st.top = Val::Px(p.y);
            }
            None => {
                st.left = Val::Px(-300.0);
            }
        }
        let a = (1.6 - f.t / 0.9 * 1.4).clamp(0.0, 1.0);
        tx.sections[0].style.color = Color::srgba(f.rgb[0], f.rgb[1], f.rgb[2], a);
    }
}

fn feed_system(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut FeedItem)>) {
    let dt = time.delta_seconds();
    for (e, mut f) in q.iter_mut() {
        f.ttl -= dt;
        if f.ttl <= 0.0 {
            commands.entity(e).despawn_recursive();
        }
    }
}

fn spin_system(time: Res<Time>, mut q: Query<&mut Transform, With<Spin>>) {
    for mut t in q.iter_mut() {
        t.rotate_y(time.delta_seconds() * 0.9);
    }
}

fn circle(g: &mut Gizmos, cx: f32, cy: f32, r: f32, c: Color) {
    let pts = (0..=48).map(|i| {
        let a = i as f32 / 48.0 * std::f32::consts::TAU;
        Vec3::new((cx + a.cos() * r) * S, 0.45, (cy + a.sin() * r) * S)
    });
    g.linestrip(pts, c);
}

fn draw_gizmos(mut gizmos: Gizmos, sess: Res<Session>) {
    let s = &*sess;
    if s.screen == Screen::Menu {
        return;
    }
    let g = &s.game;
    let p = g.player();
    let (px, py) = (g.units[p].x, g.units[p].y);

    // дальность башен рядом с героем
    for u in g.units.iter().filter(|u| u.kind == Kind::Tower && !u.dead) {
        if dist(px, py, u.x, u.y) < u.range + 450.0 {
            let c = TEAM_COL[u.team];
            circle(
                &mut gizmos,
                u.x,
                u.y,
                u.range,
                Color::srgba(c[0], c[1], c[2], 0.5),
            );
        }
    }
    // метеор и буря
    for z in g.zones.iter() {
        circle(&mut gizmos, z.x, z.y, z.r, col(z.col));
    }
    // руна
    if let Some((rx, ry)) = g.rune {
        let gold = Color::srgb(0.89, 0.70, 0.29);
        circle(&mut gizmos, rx, ry, 28.0, gold);
        circle(&mut gizmos, rx, ry, 18.0, gold);
        gizmos.line(
            Vec3::new(rx * S, 0.4, ry * S),
            Vec3::new(rx * S, 4.0, ry * S),
            gold,
        );
    }
    // цель под курсором
    if let Some(j) = s.hover.and_then(|id| g.idx(id)) {
        let u = &g.units[j];
        circle(
            &mut gizmos,
            u.x,
            u.y,
            u.r + 12.0,
            Color::srgb(1.0, 0.42, 0.42),
        );
    }
    // прицел способности
    if let Some(i) = s.aim {
        if !g.units[p].dead {
            let a = HEROES[g.hd(p).key].abilities[i];
            let c = col(HEROES[g.hd(p).key].color);
            let d = s.mouse - Vec2::new(px, py);
            let len = d.length().max(0.001);
            let dir = d / len;
            if a.ty == AbType::Skillshot {
                let n = Vec2::new(-dir.y, dir.x) * a.width;
                let e = Vec2::new(px, py) + dir * a.range;
                for sgn in [-1.0f32, 1.0] {
                    let o = n * sgn;
                    gizmos.line(
                        Vec3::new((px + o.x) * S, 0.45, (py + o.y) * S),
                        Vec3::new((e.x + o.x) * S, 0.45, (e.y + o.y) * S),
                        c,
                    );
                }
                circle(&mut gizmos, e.x, e.y, a.width, c);
            } else {
                circle(
                    &mut gizmos,
                    px,
                    py,
                    a.range.max(a.radius),
                    Color::srgba(1.0, 1.0, 1.0, 0.35),
                );
                if a.range > 0.0 {
                    let k = len.min(a.range);
                    let t = Vec2::new(px, py) + dir * k;
                    circle(&mut gizmos, t.x, t.y, a.radius.max(20.0), c);
                }
            }
        }
    }
}
