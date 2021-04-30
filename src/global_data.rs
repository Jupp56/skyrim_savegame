/// All types used in the global data tables

use crate::SaveFileReader;
use crate::fundamental_types::*;
use std::convert::TryInto;
use anyhow::Error;
use crate::reader::{read_ref_id, read_vsval_to_u32, read_ref_ids_into_vec, read_into_vec, read_u32s_into_vec};
use std::fmt::{Debug, Formatter};

trait Parse {
    fn parse(r: &mut SaveFileReader) -> GlobalDataType;
}

/// Reads and parses global data into a Vec beginning at the current index of the provided SaveFileReader.
/// This method relies on there actually being a global data structure at the indicated position.
/// It currently may panic on finding another structure due to checked buffer overflows.
pub fn read_global_data(r: &mut SaveFileReader, count: u32) -> Vec<GlobalDataType> {
    read_into_vec(r, count, |sfr| {
        let data_type = sfr.read_u32();
        let length = sfr.read_u32();
        let data: Vec<u8> = sfr.read_bytes_to_vec(length as usize);
        read_global_data_type(data_type, length, data).unwrap()
    })
}

fn read_global_data_type(data_type: u32, _data_length: u32, data: Vec<u8>) -> Result<GlobalDataType, Error> {
    let mut r = SaveFileReader::new(data);

    match data_type {
        0 => Ok(MiscStats::parse(&mut r)),
        1 => Ok(GlobalDataType::PlayerLocation(read_player_location(&mut r))),
        2 => Ok(GlobalDataType::TES(read_tes(&mut r))),
        3 => Ok(GlobalDataType::GlobalVariables(read_global_variables(&mut r))),
        4 => Ok(GlobalDataType::CreatedObjects(read_created_objects(&mut r))),
        5 => Ok(GlobalDataType::Effects(read_effects(&mut r))),
        6 => Ok(GlobalDataType::Weather(read_weather(&mut r))),
        7 => Ok(GlobalDataType::Audio(read_audio(&mut r))),
        8 => Ok(GlobalDataType::SkyCells(read_sky_cells(&mut r))),
        100 => Ok(GlobalDataType::ProcessLists(read_process_lists(&mut r))),
        101 => Ok(GlobalDataType::Combat(r.get_buffer())),
        102 => Ok(GlobalDataType::Interface(read_interface(&mut r))),
        103 => Ok(ActorCauses::parse(&mut r)),
        104 => Ok(GlobalDataType::Unknown104(r.get_buffer())),
        105 => Ok(DetectionManagerUnknown0::parse(&mut r)),
        106 => Ok(LocationMetaDataUnknown0::parse(&mut r)),
        107 => Ok(QuestStaticData::parse(&mut r)),
        108 => Ok(GlobalDataType::StoryTeller(r.read_u8() != 0)),
        109 => Ok(MagicFavorites::parse(&mut r)),
        110 => Ok(GlobalDataType::PlayerControls((r.read_u8(), r.read_u8(), r.read_u8(), r.read_u16(), r.read_u8()))),
        111 => Ok(StoryEventManager::parse(&mut r)),
        112 => Ok(IngredientsCombined::parse(&mut r)),
        113 => Ok(GlobalDataType::MenuControls((r.read_u8(), r.read_u8()))),
        114 => Ok(GlobalDataType::MenuTopicManager((read_ref_id(&mut r), read_ref_id(&mut r)))),
        1000 => Ok(GlobalDataType::TempEffects(r.get_buffer())),
        1001 => Ok(GlobalDataType::Papyrus(r.get_buffer())),
        1002 => Ok(AnimObject::parse(&mut r)),
        1003 => Ok(GlobalDataType::Timer((r.read_u8(), r.read_u8()))),
        1004 => Ok(GlobalDataType::SynchronizedAnimations(r.get_buffer())),
        1005 => Ok(GlobalDataType::Main),
        _ => {
            println!("Found unknown global data type!");
            Ok(GlobalDataType::Main)
        }
    }
}


#[derive(Clone, Debug)]
pub enum GlobalDataType {
    MiscStats(Vec<MiscStats>),
    PlayerLocation(PlayerLocation),
    TES(TES),
    GlobalVariables(Vec<GlobalVariable>),
    CreatedObjects(CreatedObjects),
    Effects(Effects),
    Weather(Weather),
    Audio(Audio),
    SkyCells(Vec<SkyCellUnknown0>),
    ProcessLists(ProcessLists),
    /// Currently not parsed
    Combat(Vec<u8>),
    Interface(Interface),
    ActorCauses(ActorCauses),
    Unknown104(Vec<u8>),
    DetectionManager(Vec<DetectionManagerUnknown0>),
    LocationMetaData(Vec<LocationMetaDataUnknown0>),
    QuestStaticData(QuestStaticData),
    /// saved as u8
    StoryTeller(bool),
    MagicFavorites(MagicFavorites),
    PlayerControls((u8, u8, u8, u16, u8)),
    StoryEventManager(StoryEventManager),
    /// Pairs of failed ingredient combinations in alchemy.
    IngredientShared(Vec<IngredientsCombined>),
    MenuControls((u8, u8)),
    MenuTopicManager((RefIdType, RefIdType)),
    /// Currently not parsed, as this is a very complicated data structure with almost no known information
    TempEffects(Vec<u8>),
    /// Currently not parsed, VERY complex
    Papyrus(Vec<u8>),
    /// Array with currently active actor reference + animation combo? Haven't yet determined when these are saved.
    AnimObjects(Vec<AnimObject>),
    /// no further known information
    Timer((u8, u8)),
    /// uesp hasn't even got a page for that
    SynchronizedAnimations(Vec<u8>),
    /// Always empty, not read by skyrim due to bug
    Main,
}

#[derive(Clone, Debug)]
pub struct MiscStats {
    pub name: String,
    pub category: MiscStatCategory,
    pub value: u32,
}

#[derive(Clone, Debug)]
pub enum MiscStatCategory {
    General,
    Quest,
    Combat,
    Magic,
    Crafting,
    Crime,
    /// DLC Stats? Showed up in Skyrim 1.7.7, but nothing shows up in the in-game menu.
    /// Four values observed:
    /// - NumVampirePerks (if Dawnguard is installed)
    /// - NumWerewolfPerks (if Dawnguard is installed)
    /// - SolstheimLocationsDiscovered (if Dragonborn is installed).
    /// - StalhrimItemsCrafted (if Dragonborn is installed).
    DLCStats,
    /// previously unobserved values, corrupted file
    Error,
}

impl Parse for MiscStats {
    fn parse(r: &mut SaveFileReader) -> GlobalDataType {
        let count = r.read_u32();
        let mut results = Vec::new();
        for _i in 0..count {
            results.push(MiscStats {
                name: r.read_w_string().content,
                category: match r.read_u8() {
                    0 => MiscStatCategory::General,
                    1 => MiscStatCategory::Quest,
                    2 => MiscStatCategory::Combat,
                    3 => MiscStatCategory::Magic,
                    4 => MiscStatCategory::Crafting,
                    5 => MiscStatCategory::Crime,
                    6 => MiscStatCategory::DLCStats,
                    _ => MiscStatCategory::Error
                },
                value: r.read_u32(),
            });
        }
        GlobalDataType::MiscStats(results)
    }
}

#[derive(Clone, Debug)]
pub struct PlayerLocation {
    /// Number of next savegame specific object id, i.e. FFxxxxxx.
    pub next_object_id: u32,
    /// This form is usually 0x0 or a worldspace. coorX and coorY represent a cell in this worldspace.
    pub world_space_1: RefIdType,
    /// x-coordinate (cell coordinates) in worldSpace1.
    pub coor_x: i32,
    /// y-coordinate (cell coordinates) in worldSpace1
    pub coor_y: i32,
    /// This can be either a worldspace or an interior cell.
    /// If it's a worldspace, the player is located at the cell (coorX, coorY).
    /// posX/Y/Z is the player's position inside the cell
    pub world_space_2: RefIdType,
    /// x-coordinate in worldSpace2
    pub pos_x: f32,
    /// y-coordinate in worldSpace2
    pub pos_y: f32,
    /// z-coordinate in worldSpace2
    pub pos_z: f32,
    /// vsval? It seems absent in 9th version. Not read by this library.
    pub unk: Vec<u8>,
}

fn read_player_location(r: &mut SaveFileReader) -> PlayerLocation {
    PlayerLocation {
        next_object_id: r.read_u32(),
        world_space_1: read_ref_id(r),
        coor_x: r.read_i32(),
        coor_y: r.read_i32(),
        world_space_2: read_ref_id(r),
        pos_x: r.read_f32(),
        pos_y: r.read_f32(),
        pos_z: r.read_f32(),
        /// we dont know what it is and it seems to be absent in some versions.
        unk: vec![],
    }
}

#[derive(Clone)]
pub struct TES {
    pub u1: Vec<TESUnknown0>,
    pub u2: Vec<RefIdType>,
    pub u3: Vec<RefIdType>,
}

#[derive(Clone, Debug)]
pub struct TESUnknown0 {
    pub form_id: RefIdType,
    pub unknown: u16,
}

impl Debug for TES {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TES")
            .field("TESUnknown0 (length)", &self.u1.len())
            .field("u1 (length)", &self.u2.len())
            .field("u2 (length)", &self.u3.len())
            .finish()
    }
}

fn read_tes(r: &mut SaveFileReader) -> TES {
    let mut u1 = Vec::new();
    for _i in 0..read_vsval_to_u32(r) {
        u1.push(TESUnknown0 {
            form_id: read_ref_id(r),
            unknown: r.read_u16(),
        })
    }
    let mut u2 = Vec::new();
    for _i in 0..r.read_u32() * 2 {
        u2.push(read_ref_id(r))
    }
    let mut u3 = Vec::new();
    for _i in 0..read_vsval_to_u32(r) {
        u3.push(read_ref_id(r))
    }
    TES {
        u1,
        u2,
        u3,
    }
}

#[derive(Clone, Debug)]
pub struct GlobalVariable {
    pub form_id: RefIdType,
    pub value: f32,
}

fn read_global_variables(r: &mut SaveFileReader) -> Vec<GlobalVariable> {
    let mut vec = Vec::new();
    for _i in 0..read_vsval_to_u32(r) {
        vec.push(GlobalVariable {
            form_id: read_ref_id(r),
            value: r.read_f32(),
        });
    }
    vec
}

#[derive(Clone, Debug)]
pub struct CreatedObjects {
    /// List of all created enchantments that are/were applied to weapons.
    pub weapon_ench_table: Vec<Enchantment>,
    /// List of all created enchantments that are/were applied to armour.
    /// Not sure which types of armour (Body/Gloves/Boots/Shield/etc) this encompasses.
    pub armour_ench_table: Vec<Enchantment>,
    /// List of all created potions.
    pub potion_table: Vec<Enchantment>,
    /// List of all created poisons.
    pub poison_table: Vec<Enchantment>,
}

fn read_created_objects(r: &mut SaveFileReader) -> CreatedObjects {
    let weapon_ench_table_count = read_vsval_to_u32(r);
    let weapon_ench_table = read_enchantments(r, weapon_ench_table_count);
    let armour_ench_table_count = read_vsval_to_u32(r);
    let armour_ench_table = read_enchantments(r, armour_ench_table_count);
    let potion_table_count = read_vsval_to_u32(r);
    let potion_table = read_enchantments(r, potion_table_count);
    let poison_table_count = read_vsval_to_u32(r);
    let poison_table = read_enchantments(r, poison_table_count);

    CreatedObjects {
        weapon_ench_table,
        armour_ench_table,
        potion_table,
        poison_table,
    }
}

#[derive(Clone, Debug)]
pub struct Enchantment {
    /// FormID of the enchantment. I've only seen created types, no default or array types.
    pub ref_id: RefIdType,
    /// Seems to represent the number of times the enchantment is used?
    /// However, sometimes using the same enchantment nets two different forms.
    /// Could just be a bug in Skyrim.
    pub times_used: u32,
    pub effects: Vec<MagicEffect>,
}

fn read_enchantments(r: &mut SaveFileReader, count: u32) -> Vec<Enchantment> {
    let mut enchantments = Vec::new();
    for _i in 0..count {
        let ref_id = read_ref_id(r);
        let times_used = r.read_u32();
        let effects_count = read_vsval_to_u32(r);
        let effects = read_magic_effects(r, effects_count);
        enchantments.push(Enchantment {
            ref_id,
            times_used,
            effects,
        });
    }
    enchantments
}

#[derive(Clone, Debug)]
pub struct MagicEffect {
    pub effect_id: RefIdType,
    pub info: EnchInfo,
    /// Amount this enchantment adds to the base item's price.
    pub price: f32,
}

fn read_magic_effects(r: &mut SaveFileReader, count: u32) -> Vec<MagicEffect> {
    read_into_vec(
        r,
        count,
        |r|
            MagicEffect {
                effect_id: read_ref_id(r),
                info: EnchInfo {
                    magnitude: r.read_f32(),
                    duration: r.read_u32(),
                    area: r.read_u32(),
                },
                price: r.read_f32(),
            })
}

#[derive(Clone, Debug)]
pub struct EnchInfo {
    pub magnitude: f32,
    pub duration: u32,
    pub area: u32,
}

#[derive(Clone, Debug)]
pub struct Effects {
    pub image_space_modifiers: Vec<Effect>,
    pub unknown1: f32,
    pub unknown2: f32,
}

fn read_effects(r: &mut SaveFileReader) -> Effects {
    let image_space_modifiers_length = read_vsval_to_u32(r);
    let mut image_space_modifiers = Vec::new();
    for _i in 0..image_space_modifiers_length {
        image_space_modifiers.push({
            Effect {
                strength: r.read_f32(),
                timestamp: r.read_f32(),
                unknown: r.read_u32(),
                effect_id: read_ref_id(r),
            }
        });
    }
    Effects {
        image_space_modifiers,
        unknown1: r.read_f32(),
        unknown2: r.read_f32(),
    }
}

#[derive(Clone, Debug)]
pub struct Effect {
    /// Value from 0 to 1 (0 is no effect, 1 is full effect)
    pub strength: f32,
    /// Time from effect beginning
    pub timestamp: f32,
    /// May be flag. Appears when you аdd a crossfade imagespace modifier to the active list with imodcf command
    pub unknown: u32,
    pub effect_id: RefIdType,
}

#[derive(Clone, Debug)]
pub struct Weather {
    pub climate: RefIdType,
    pub weather: RefIdType,
    /// Only during weather transition. In other cases it equals zero.
    pub prev_weather: RefIdType,
    pub unk_weather_1: RefIdType,
    pub unk_weather_2: RefIdType,
    pub regn_weather: RefIdType,
    /// Current in-game time in hours
    pub cur_time: f32,
    /// Time of current weather beginning
    pub beg_time: f32,
    /// A value from 0.0 to 1.0 describing how far in the current weather has transitioned
    pub weather_pct: f32,
    pub u1: u32,
    pub u2: u32,
    pub u3: u32,
    pub u4: u32,
    pub u5: u32,
    pub u6: u32,
    pub u7: f32,
    pub u8: u32,
    pub flags: u8,
    /// Unresearched format. Only present if flags has bit 0 set.
    pub u9: Option<String>,
    /// Unresearched format. Only present if flags has bit 1 set.
    pub u10: Option<String>,
}

fn read_weather(r: &mut SaveFileReader) -> Weather {
    let climate = read_ref_id(r);
    let weather = read_ref_id(r);
    let prev_weather = read_ref_id(r);
    let unk_weather_1 = read_ref_id(r);
    let unk_weather_2 = read_ref_id(r);
    let regn_weather = read_ref_id(r);
    let cur_time = r.read_f32();
    let beg_time = r.read_f32();
    let weather_pct = r.read_f32();
    let u1 = r.read_u32();
    let u2 = r.read_u32();
    let u3 = r.read_u32();
    let u4 = r.read_u32();
    let u5 = r.read_u32();
    let u6 = r.read_u32();
    let u7 = r.read_f32();
    let u8 = r.read_u32();
    let flags = r.read_u8();
    let mut u9 = None;
    let mut u10 = None;
    if flags & 0b10000000 == 0b10000000 {
        u9 = Some("Unbekannter Datentyp".to_string())
    }
    if flags & 0b01000000 == 0b01000000 {
        u10 = Some("Unbekannter Datentyp".to_string())
    }
    Weather {
        climate,
        weather,
        prev_weather,
        unk_weather_1,
        unk_weather_2,
        regn_weather,
        cur_time,
        beg_time,
        weather_pct,
        u1,
        u2,
        u3,
        u4,
        u5,
        u6,
        u7,
        u8,
        flags,
        u9,
        u10,
    }
}

#[derive(Clone, Debug)]
pub struct Audio {
    /// Only the UIActivateFail sound descriptor has been observed here.
    pub unknown: RefIdType,
    /// Seems to contain music tracks (MUST records) that were playing at the time of saving, not including the background music
    pub tracks: Vec<RefIdType>,
    /// Background music at time of saving
    pub bgm: RefIdType,
}

pub fn read_audio(r: &mut SaveFileReader) -> Audio {
    let unknown = read_ref_id(r);
    let tracks_count = read_vsval_to_u32(r);
    let tracks = read_ref_ids_into_vec(r, tracks_count);
    let bgm = read_ref_id(r);
    Audio {
        unknown,
        tracks,
        bgm,
    }
}

#[derive(Clone, Debug)]
pub struct SkyCellUnknown0 {
    pub u1: RefIdType,
    pub u2: RefIdType,
}

fn read_sky_cells(r: &mut SaveFileReader) -> Vec<SkyCellUnknown0> {
    let count = read_vsval_to_u32(r);
    read_into_vec(r, count, |r| SkyCellUnknown0 {
        u1: read_ref_id(r),
        u2: read_ref_id(r),
    })
}

#[derive(Clone, Debug)]
pub struct ProcessLists {
    pub u1: f32,
    pub u2: f32,
    pub u3: f32,
    /// This value is assigned to the next process
    pub next_num: u32,
    /// Crimes grouped according with their type (see below)
    pub all_crimes: Vec<Crime>,
}

fn read_process_lists(r: &mut SaveFileReader) -> ProcessLists {
    let u1 = r.read_f32();
    let u2 = r.read_f32();
    let u3 = r.read_f32();
    let next_num = r.read_u32();
    let crime_type_count = read_vsval_to_u32(r);
    let all_crimes = read_into_vec(r, crime_type_count, |r| {
        read_crime(r)
    });
    ProcessLists {
        u1,
        u2,
        u3,
        next_num,
        all_crimes,
    }
}

#[derive(Clone, Debug)]
pub struct Crime {
    pub witness_num: u32,
    pub crime_type: CrimeType,
    pub u1: u8,
    /// The number of stolen items (e.g. if you've stolen Gold(7), it would be equals to 7).
    ///Only for thefts
    pub quantity: u32,
    /// Assigned in accordance with nextNum
    pub serial_num: u32,
    pub u2: u8,
    /// May be date of crime? Little byte is equal to day
    pub u3: u32,
    /// Negative value measured from moment of crime
    pub elapsed_time: f32,
    /// The killed, forced door, stolen item etc.
    pub victim_id: RefIdType,
    pub criminal_id: RefIdType,
    /// Only for thefts
    pub item_base_id: RefIdType,
    /// Faction, outfit etc. Only for thefts
    pub ownership_id: RefIdType,
    pub witnesses: Vec<RefIdType>,
    pub bounty: u32,
    pub crime_faction_id: RefIdType,
    /// 0 - active crime, 1 - it was atoned
    pub is_cleared: bool,
    pub u4: u16,
}

fn read_crime(r: &mut SaveFileReader) -> Crime {
    let witness_num = r.read_u32();
    let crime_type = convert_to_crime_type(r.read_u32());
    let u1 = r.read_u8();
    let quantity = r.read_u32();
    let serial_num = r.read_u32();
    let u2 = r.read_u8();
    let u3 = r.read_u32();
    let elapsed_time = r.read_f32();
    let victim_id = read_ref_id(r);
    let criminal_id = read_ref_id(r);
    let item_base_id = read_ref_id(r);
    let ownership_id = read_ref_id(r);
    let count = read_vsval_to_u32(r);
    let witnesses = read_into_vec(r, count, |r| read_ref_id(r));
    let bounty = r.read_u32();
    let crime_faction_id = read_ref_id(r);
    let is_cleared = match r.read_u8() {
        0 => false,
        1 => true,
        _ => {
            println!("Found new value for isCleared on crime! Please report that and attach your savegame.");
            true
        }
    };
    let u4 = r.read_u16();
    Crime {
        witness_num,
        crime_type,
        u1,
        quantity,
        serial_num,
        u2,
        u3,
        elapsed_time,
        victim_id,
        criminal_id,
        item_base_id,
        ownership_id,
        witnesses,
        bounty,
        crime_faction_id,
        is_cleared,
        u4,
    }
}

#[derive(Clone, Debug)]
pub enum CrimeType {
    Theft,
    Pickpocketing,
    Trespassing,
    Assault,
    Murder,
    Unknown5,
    Lycanthropy,
    Error,
}

fn convert_to_crime_type(num: u32) -> CrimeType {
    match num {
        0 => CrimeType::Theft,
        1 => CrimeType::Pickpocketing,
        2 => CrimeType::Trespassing,
        3 => CrimeType::Assault,
        4 => CrimeType::Murder,
        5 => CrimeType::Unknown5,
        6 => CrimeType::Lycanthropy,
        _ => {
            println!("Encountered unknown crimeType");
            CrimeType::Error
        }
    }
}

#[derive(Clone, Debug)]
pub struct Interface {
    /// - 0xEC - HelpLockpickingShort
    /// - 0xEE - HelpSmithingShort
    /// - 0xEF - HelpCookingPots
    /// - 0xF0 - HelpSmeltingShort
    /// - 0xF1 - HelpTanningShort
    /// - 0xF3 - HelpEnchantingShort
    /// - 0xF4 - HelpGrindstoneShort
    /// - 0xF5 - HelpArmorBenchShort
    /// - 0xF6 - HelpAlchemyShort
    /// - 0xF7 - HelpBarterShortPC
    /// - 0xF9 - HelpLevelingShort
    /// - 0xFA - HelpWorldMapShortPC
    /// - 0xFB - HelpJournalShortPC
    /// - 0xFF - HelpJailTutorial
    /// - 0x100 - HelpFollowerCommandTutorial
    /// - 0x102 - HelpFavoritesPCShort
    ///
    ///etc.
    pub shown_help_msg: Vec<u32>,
    pub u0: u8,
    pub last_used_weapons: Vec<RefIdType>,
    pub last_used_spells: Vec<RefIdType>,
    pub last_used_shouts: Vec<RefIdType>,
    pub u1: u8,
    /// This value is only present in certain situations. Undetermined when.
    /// Library currently does not parse this => always None
    pub u2: Option<InterfaceUnknown0>,
}

fn read_interface(r: &mut SaveFileReader) -> Interface {
    let shown_help_message_count = r.read_u32();
    let shown_help_msg = read_u32s_into_vec(r, shown_help_message_count);
    let u0 = r.read_u8();
    let last_used_weapons_count = read_vsval_to_u32(r);
    let last_used_weapons = read_ref_ids_into_vec(r, last_used_weapons_count);
    let last_used_spells_count = read_vsval_to_u32(r);
    let last_used_spells = read_ref_ids_into_vec(r, last_used_spells_count);
    let last_used_shouts_count = read_vsval_to_u32(r);
    let last_used_shouts = read_ref_ids_into_vec(r, last_used_shouts_count);
    let u1 = r.read_u8();
    // This value is only there sometimes. Rather not risk overflowing the buffer.
    let u2 = None;
    Interface {
        shown_help_msg,
        u0,
        last_used_weapons,
        last_used_spells,
        last_used_shouts,
        u1,
        u2,
    }
}

#[derive(Clone, Debug)]
pub struct InterfaceUnknown0 {
    pub unknown_0_0: Vec<InterfaceUnknown0_0>,
    pub unknown1: Vec<String>,
    pub unknown2: u32,
}


#[derive(Clone, Debug)]
pub struct InterfaceUnknown0_0 {
    u0: String,
    u1: String,
    u2: u32,
    u3: u32,
    u4: u32,
    u5: u32,
}

#[derive(Clone, Debug)]
pub struct ActorCauses {
    next_num: u32,
    unknown: Vec<ActorCausesUnknown0>,
}

impl Parse for ActorCauses {
    fn parse(r: &mut SaveFileReader) -> GlobalDataType {
        let next_num = r.read_u32();
        let count = read_vsval_to_u32(r);
        let unknown = read_into_vec(r, count, |r| {
            ActorCausesUnknown0 {
                x: r.read_f32(),
                y: r.read_f32(),
                z: r.read_f32(),
                serial_num: r.read_u32(),
                actor_id: read_ref_id(r),
            }
        });
        GlobalDataType::ActorCauses(ActorCauses {
            next_num,
            unknown,
        })
    }
}

#[derive(Clone, Debug)]
pub struct ActorCausesUnknown0 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub serial_num: u32,
    pub actor_id: RefIdType,
}

#[derive(Clone, Debug)]
pub struct DetectionManagerUnknown0 {
    pub u0: RefIdType,
    pub u1: u32,
    pub u2: u32,
}

impl Parse for DetectionManagerUnknown0 {
    fn parse(r: &mut SaveFileReader) -> GlobalDataType {
        let count = read_vsval_to_u32(r);
        GlobalDataType::DetectionManager(
            read_into_vec(
                r,
                count,
                |r| DetectionManagerUnknown0 {
                    u0: read_ref_id(r),
                    u1: r.read_u32(),
                    u2: r.read_u32(),
                }))
    }
}

#[derive(Clone, Debug)]
pub struct LocationMetaDataUnknown0 {
    pub u0: RefIdType,
    pub u1: u32,
}

impl Parse for LocationMetaDataUnknown0 {
    fn parse(r: &mut SaveFileReader) -> GlobalDataType {
        let count = read_vsval_to_u32(r);
        GlobalDataType::LocationMetaData(
            read_into_vec(
                r,
                count,
                |r| LocationMetaDataUnknown0 {
                    u0: read_ref_id(r),
                    u1: r.read_u32(),
                }))
    }
}

#[derive(Clone, Debug)]
pub struct QuestStaticData {
    pub u0: Vec<QuestRunDataItem3>,
    pub u1: Vec<QuestRunDataItem3>,
    pub u2: Vec<RefIdType>,
    pub u3: Vec<RefIdType>,
    pub u4: Vec<RefIdType>,
    pub u5: Vec<QuestStaticDataUnknown0>,
    pub u6: u8,
}

impl Parse for QuestStaticData {
    fn parse(r: &mut SaveFileReader) -> GlobalDataType {
        let count = r.read_u32();
        let u0 = read_into_vec(r, count, |r| read_quest_run_data_item_3(r));
        let count1 = r.read_u32();
        let u1 = read_into_vec(r, count1, |r| read_quest_run_data_item_3(r));
        let count2 = r.read_u32();
        let u2 = read_ref_ids_into_vec(r, count2);
        let count3 = r.read_u32();
        let u3 = read_ref_ids_into_vec(r, count3);
        let count4 = r.read_u32();
        let u4 = read_ref_ids_into_vec(r, count4);
        let count5 = read_vsval_to_u32(r);
        let u5 = read_into_vec(r, count5, |r| read_quest_static_data_unknown_0(r));
        let u6 = r.read_u8();

        GlobalDataType::QuestStaticData(QuestStaticData {
            u0,
            u1,
            u2,
            u3,
            u4,
            u5,
            u6,
        })
    }
}

#[derive(Clone, Debug)]
pub struct QuestRunDataItem3 {
    pub u1: u32,
    pub u2: f32,
    pub quest_run_data_item_3_data: Vec<QuestRunDataItem3DataType>,
}

fn read_quest_run_data_item_3(r: &mut SaveFileReader) -> QuestRunDataItem3 {
    let u1 = r.read_u32();
    let u2 = r.read_f32();
    let count = r.read_u32();
    let quest_run_data_item_3_data = read_into_vec(r, count, |r| read_quest_run_data_item_3_data_type(r));
    QuestRunDataItem3 {
        u1,
        u2,
        quest_run_data_item_3_data,
    }
}


#[derive(Clone, Debug)]
pub enum QuestRunDataItem3DataType {
    RefId(RefIdType),
    U32(u32),
}

fn read_quest_run_data_item_3_data_type(r: &mut SaveFileReader) -> QuestRunDataItem3DataType {
    let data_type = r.read_u32();
    match data_type {
        3 => QuestRunDataItem3DataType::U32(r.read_u32()),
        1 | 2 | 4 => QuestRunDataItem3DataType::RefId(read_ref_id(r)),
        _ => {
            println!("Encountered unknown questrundataitem3 type. Assuming refId");
            QuestRunDataItem3DataType::RefId(read_ref_id(r))
        }
    }
}

#[derive(Clone, Debug)]
pub struct QuestStaticDataUnknown0 {
    pub unk0_0: RefIdType,
    pub u1: Vec<QuestStaticDataUnknown1>,
}

fn read_quest_static_data_unknown_0(r: &mut SaveFileReader) -> QuestStaticDataUnknown0 {
    let unk0_0 = read_ref_id(r);
    let count = read_vsval_to_u32(r);
    let u1 = read_into_vec(r, count, |r| QuestStaticDataUnknown1 {
        unk_1_0: r.read_u32(),
        unk_1_1: r.read_u32(),
    });
    QuestStaticDataUnknown0 {
        unk0_0,
        u1,
    }
}

#[derive(Clone, Debug)]
pub struct QuestStaticDataUnknown1 {
    pub unk_1_0: u32,
    pub unk_1_1: u32,
}

#[derive(Clone, Debug)]
pub struct MagicFavorites {
    /// Spells, shouts, abilities etc.
    pub favorited_magics: Vec<RefIdType>,
    /// Hotkey corresponds to the position of magic in this array
    pub magic_hot_keys: Vec<RefIdType>,
}

impl Parse for MagicFavorites {
    fn parse(r: &mut SaveFileReader) -> GlobalDataType {
        let count0 = read_vsval_to_u32(r);
        let favorited_magics = read_ref_ids_into_vec(r, count0);
        let count1 = read_vsval_to_u32(r);
        let magic_hot_keys = read_ref_ids_into_vec(r, count1);
        GlobalDataType::MagicFavorites(MagicFavorites {
            favorited_magics,
            magic_hot_keys,
        })
    }
}

#[derive(Clone, Debug)]
pub struct StoryEventManager {
    pub u0: u32,
    /// Unknown format. Possibly the same as unk0 and unk1 in Quest Static Data
    /// Vector represents there is a list. Currently just capacity adjusted.
    pub u1: Vec<u8>,
}

impl Parse for StoryEventManager {
    fn parse(r: &mut SaveFileReader) -> GlobalDataType {
        let u0 = r.read_u32();
        let count = read_vsval_to_u32(r);

        GlobalDataType::StoryEventManager(StoryEventManager {
            u0,
            u1: Vec::with_capacity(match count.try_into() {
                Ok(x) => x,
                Err(_) => usize::max_value()
            }),
        })
    }
}

#[derive(Clone, Debug)]
pub struct IngredientsCombined {
    pub ingredient0: RefIdType,
    pub ingredient1: RefIdType,
}

impl Parse for IngredientsCombined {
    fn parse(r: &mut SaveFileReader) -> GlobalDataType {
        let count = r.read_u32();
        GlobalDataType::IngredientShared(
            read_into_vec(
                r,
                count,
                |r| IngredientsCombined {
                    ingredient0: read_ref_id(r),
                    ingredient1: read_ref_id(r),
                }))
    }
}

#[derive(Clone, Debug)]
pub struct AnimObject {
    /// RefID pointing to an actor reference.
    pub achr: RefIdType,
    /// RefID pointing to an animation form.
    pub anim: RefIdType,
    /// Unknown but only 0 and 1 have been observed.
    pub u1: u8,
}

impl Parse for AnimObject {
    fn parse(r: &mut SaveFileReader) -> GlobalDataType {
        let count = r.read_u32();
        GlobalDataType::AnimObjects(read_into_vec(r, count, |r| AnimObject {
            achr: read_ref_id(r),
            anim: read_ref_id(r),
            u1: r.read_u8(),
        }))
    }
}