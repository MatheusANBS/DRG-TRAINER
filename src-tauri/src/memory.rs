use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::ffi::c_void;
use std::fs::{self, File};
use std::io::Read;
use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, UNIX_EPOCH};
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Diagnostics::Debug::{ReadProcessMemory, WriteProcessMemory};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, PROCESSENTRY32W,
    Process32FirstW, Process32NextW, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READWRITE, VirtualAllocEx, VirtualFreeEx,
};
use windows::Win32::System::Threading::{
    CreateRemoteThread, GetExitCodeThread, OpenProcess, PROCESS_CREATE_THREAD,
    PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
    WaitForSingleObject,
};

const PROCESS_NAME: &str = "FSD-Win64-Shipping.exe";
const EXPECTED_EXE_SHA256: &str =
    "8e22e3710c607e811e0319a4378d2b7ab2e46c6b46406629a6657b1662c77a41";
const GUOBJECT_ARRAY_RVA: usize = 0x0660_1040;
const FNAME_POOL_RVA: usize = 0x065C_4A80;
const CREDITS_OFFSET: usize = 0x590;
const RESOURCES_SAVE_OFFSET: usize = 0x8E8;
const RESOURCE_SAVEGAME_ID_OFFSET: usize = 0xE0;
const RESOURCE_MAP_ELEMENT_SIZE: usize = 0x1C;
const CHARACTER_SAVES_OFFSET: usize = 0x580;
const CHARACTER_SAVE_SIZE: usize = 0x2F8;
const CHARACTER_XP_OFFSET: usize = 0x10;
const CHARACTER_PROMOTIONS_OFFSET: usize = 0x18;
const OWNED_PERKS_OFFSET: usize = 0x160;
const PURCHASED_ITEM_UPGRADES_OFFSET: usize = 0x620;
const UNLOCKED_ITEMS_OFFSET: usize = 0x630;
const OWNED_ITEMS_OFFSET: usize = 0x640;
const SCHEMATIC_SAVE_OFFSET: usize = 0x218;
const FORGED_SCHEMATICS_OFFSET: usize = 0x18;
const OWNED_SCHEMATICS_OFFSET: usize = 0x28;
const SCHEMATIC_SAVEGAME_ID_OFFSET: usize = 0x3C;
const SCHEMATIC_SETTINGS_ALL_SCHEMATICS_OFFSET: usize = 0x348;
const TSET_ALLOCATION_FLAGS_OFFSET: usize = 0x20;
const TSET_MAX_INDEX_OFFSET: usize = 0x28;
const TSET_ELEMENT_SIZE: usize = 0x10;
const SCHEMATIC_ITEM_OFFSET: usize = 0xA8;
const FSD_SAVEGAME_CLASS_INDEX: u32 = 1662;
const ACTIVE_SAVEGAME_INDEX: u32 = 40754;

// Native implementations resolved from the reflected UFunctions in the verified build.
const UNLOCK_ALL_WEAPONS_RVA: usize = 0x0169_1540;
const UNLOCK_ALL_PERKS_RVA: usize = 0x0168_E670;
const UNLOCK_ALL_UPGRADES_RVA: usize = 0x0169_1300;
const FORGE_SCHEMATIC_SAVE_RVA: usize = 0x016D_1660;
const OVERCLOCK_REWARD_RVA: usize = 0x0180_5EB0;
const SKIN_REWARD_RVA: usize = 0x0180_60A0;
const VANITY_REWARD_RVA: usize = 0x0180_6190;
const VICTORY_POSE_REWARD_RVA: usize = 0x0180_61B0;
const RETIRE_CHARACTER_RVA: usize = 0x016E_BB10;
const SAVE_TO_DISK_RVA: usize = 0x016E_C0F0;
const ADD_RESOURCE_RVA: usize = 0x016C_63D0;
const UNLOCK_ALL_WEAPONS_PREFIX: &[u8] = &[
    0x41, 0x54, 0x41, 0x57, 0x48, 0x81, 0xEC, 0x88, 0x00, 0x00, 0x00, 0x48, 0x8B, 0xD1,
];
const SAVE_TO_DISK_PREFIX: &[u8] = &[
    0x48, 0x83, 0xEC, 0x28, 0x80, 0xB9, 0x68, 0x0C, 0x00, 0x00, 0x00,
];
const ADD_RESOURCE_PREFIX: &[u8] = &[
    0x48, 0x89, 0x74, 0x24, 0x18, 0x57, 0x48, 0x83, 0xEC, 0x40, 0x0F, 0x29, 0x74, 0x24, 0x30,
];
const UNLOCK_ALL_PERKS_PREFIX: &[u8] = &[
    0x48, 0x89, 0x6C, 0x24, 0x18, 0x56, 0x48, 0x83, 0xEC, 0x20, 0x48, 0x8B, 0xF1,
];
const UNLOCK_ALL_UPGRADES_PREFIX: &[u8] = &[
    0x40, 0x55, 0x41, 0x54, 0x48, 0x81, 0xEC, 0x98, 0x00, 0x00, 0x00, 0x48, 0x8B, 0xD1,
];
const FORGE_SCHEMATIC_SAVE_PREFIX: &[u8] = &[
    0x48, 0x89, 0x5C, 0x24, 0x08, 0x48, 0x89, 0x6C, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x18, 0x57,
    0x48, 0x83, 0xEC, 0x20,
];
const OVERCLOCK_REWARD_PREFIX: &[u8] = &[
    0x48, 0x89, 0x6C, 0x24, 0x10, 0x48, 0x89, 0x74, 0x24, 0x18, 0x57, 0x48, 0x83, 0xEC, 0x70, 0x48,
    0x8B, 0xFA,
];
const SKIN_REWARD_PREFIX: &[u8] = &[
    0x48, 0x89, 0x5C, 0x24, 0x18, 0x48, 0x89, 0x74, 0x24, 0x20, 0x57, 0x48, 0x83, 0xEC, 0x20, 0x49,
    0x8B, 0xF0,
];
const VANITY_REWARD_PREFIX: &[u8] = &[0x49, 0x8B, 0xC0, 0x4C, 0x8B, 0x41, 0x28, 0x4D, 0x85, 0xC0];
const VICTORY_POSE_REWARD_PREFIX: &[u8] =
    &[0x49, 0x8B, 0xC0, 0x4C, 0x8B, 0x41, 0x28, 0x4D, 0x85, 0xC0];
const RETIRE_CHARACTER_PREFIX: &[u8] = &[
    0x40, 0x55, 0x56, 0x41, 0x56, 0x48, 0x83, 0xEC, 0x30, 0x48, 0x8B, 0x42, 0x30,
];
const REMOTE_CALL_TIMEOUT_MS: u32 = 15_000;
const CURRENT_WEAPON_COUNT: usize = 26;
const CURRENT_OVERCLOCK_SCHEMATIC_COUNT: usize = 160;
const CURRENT_COSMETIC_SCHEMATIC_COUNT: usize = 348;
const MAX_CLASS_XP: i32 = 315_000;
const MAX_RESOURCE_DELTA: i32 = 1_000_000;
const MAX_RESOURCE_TOTAL: f32 = 100_000_000.0;

const PLAYABLE_CLASS_IDS: [(&str, [u8; 16]); 4] = [
    (
        "Driller",
        [
            0x9E, 0xDD, 0x56, 0xF1, 0xEE, 0xBC, 0xC5, 0x48, 0x8D, 0x5B, 0x5E, 0x5B, 0x80, 0xB6,
            0x2D, 0xB4,
        ],
    ),
    (
        "Engineer",
        [
            0x85, 0xEF, 0x62, 0x6C, 0x65, 0xF1, 0x02, 0x4A, 0x8D, 0xFE, 0xB5, 0xD0, 0xF3, 0x90,
            0x9D, 0x2E,
        ],
    ),
    (
        "Gunner",
        [
            0xAE, 0x56, 0xE1, 0x80, 0xFE, 0xC0, 0xC4, 0x4D, 0x96, 0xFA, 0x29, 0xC2, 0x83, 0x66,
            0xB9, 0x7B,
        ],
    ),
    (
        "Scout",
        [
            0x30, 0xD8, 0xEA, 0x17, 0xD8, 0xFB, 0xBA, 0x4C, 0x95, 0x30, 0x6D, 0xE9, 0x65, 0x5C,
            0x2F, 0x8C,
        ],
    ),
];

const OBJECTS_MEMBER_OFFSET: usize = 0x10;
const NUM_ELEMENTS_OFFSET: usize = 0x24;
const UOBJECT_ITEM_SIZE: usize = 0x18;
const OBJECTS_PER_CHUNK: u32 = 65_536;
const UOBJECT_INTERNAL_INDEX_OFFSET: usize = 0x0C;
const UOBJECT_CLASS_OFFSET: usize = 0x10;
const UOBJECT_FNAME_OFFSET: usize = 0x18;
const UOBJECT_FNAME_NUMBER_OFFSET: usize = 0x1C;
const USTRUCT_SUPER_OFFSET: usize = 0x40;
const USTRUCT_CHILD_PROPERTIES_OFFSET: usize = 0x50;
const UCLASS_DEFAULT_OBJECT_OFFSET: usize = 0x118;
const FFIELD_NEXT_OFFSET: usize = 0x20;
const FFIELD_FNAME_OFFSET: usize = 0x28;
const FPROPERTY_ELEMENT_SIZE_OFFSET: usize = 0x3C;
const FPROPERTY_OFFSET_INTERNAL_OFFSET: usize = 0x4C;

const PLAYER_CONTROLLER_NAMES: [&str; 2] = [
    "BP_PlayerController_SpaceRig_C",
    "BP_NetworkPlayerController_C",
];
const AMMO_WEAPON_CLASS_NAME: &str = "AmmoDrivenWeapon";
const CONTROLLER_PAWN_OFFSET: usize = 0x250;
const PLAYER_INVENTORY_OFFSET: usize = 0xB18;
const INVENTORY_EQUIPPED_ACTOR_OFFSET: usize = 0xE0;
const CLIP_SIZE_OFFSET: usize = 0x684;
const CLIP_COUNT_OFFSET: usize = 0x6A0;
const DAMAGE_VALUE: f32 = 9_999.0;
const FNAME_BLOCK_SIZE: usize = 0x20_000;
const FREEZE_INTERVAL: Duration = Duration::from_millis(35);
const RECONNECT_INTERVAL: Duration = Duration::from_millis(500);

type BuildCache = Option<(PathBuf, u64, u128)>;
static BUILD_CACHE: OnceLock<Mutex<BuildCache>> = OnceLock::new();
static ACTIVE_SAVE_CACHE: OnceLock<Mutex<Option<(u32, usize)>>> = OnceLock::new();
static CLIP_CONTROLLER_CACHE: OnceLock<Mutex<Option<(u32, usize)>>> = OnceLock::new();
static CONTROLLER_FNAME_CACHE: OnceLock<Mutex<Option<(u32, Vec<u32>)>>> = OnceLock::new();
static CLIP_STATUS: OnceLock<Mutex<ClipStatus>> = OnceLock::new();
static DAMAGE_STATUS: OnceLock<Mutex<DamageStatus>> = OnceLock::new();
static INFINITE_MAGAZINE_ENABLED: AtomicBool = AtomicBool::new(false);
static CLIP_WORKER_RUNNING: AtomicBool = AtomicBool::new(false);
static WEAPON_DAMAGE_ENABLED: AtomicBool = AtomicBool::new(false);
static DAMAGE_WORKER_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
pub struct MemoryError(String);

impl std::fmt::Display for MemoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for MemoryError {}

impl From<std::io::Error> for MemoryError {
    fn from(error: std::io::Error) -> Self {
        Self(error.to_string())
    }
}

type Result<T> = std::result::Result<T, MemoryError>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditSnapshot {
    pub credits: i32,
    pub pid: u32,
    pub address: String,
    pub module_base: String,
    pub offset: String,
    pub build_valid: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteResult {
    pub previous: i32,
    pub current: i32,
    pub pid: u32,
    pub address: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSnapshot {
    pub id: String,
    pub name: String,
    pub category: String,
    pub amount: f32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceWriteResult {
    pub pid: u32,
    pub amount_added: i32,
    pub resources: Vec<ResourceSnapshot>,
    pub backup_path: String,
    pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockAllResult {
    pub pid: u32,
    pub weapon_count: usize,
    pub unlocked_before: i32,
    pub unlocked_after: i32,
    pub owned_before: i32,
    pub owned_after: i32,
    pub backup_path: String,
    pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermanentUnlockResult {
    pub pid: u32,
    pub items_before: i32,
    pub items_after: i32,
    pub backup_path: String,
    pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchematicUnlockResult {
    pub pid: u32,
    pub overclocks_processed: usize,
    pub cosmetics_processed: usize,
    pub forged_before: i32,
    pub forged_after: i32,
    pub owned_before: i32,
    pub owned_after: i32,
    pub backup_path: String,
    pub message: String,
}

#[derive(Clone, Copy)]
enum SchematicRewardKind {
    Overclock,
    Skin,
    Vanity,
    VictoryPose,
}

impl SchematicRewardKind {
    fn routine(self, module_base: usize) -> (usize, &'static [u8], &'static str) {
        match self {
            Self::Overclock => (
                module_base + OVERCLOCK_REWARD_RVA,
                OVERCLOCK_REWARD_PREFIX,
                "OverclockShematicItem::GrantReward",
            ),
            Self::Skin => (
                module_base + SKIN_REWARD_RVA,
                SKIN_REWARD_PREFIX,
                "SkinSchematicItem::GrantReward",
            ),
            Self::Vanity => (
                module_base + VANITY_REWARD_RVA,
                VANITY_REWARD_PREFIX,
                "VanitySchematicItem::GrantReward",
            ),
            Self::VictoryPose => (
                module_base + VICTORY_POSE_REWARD_RVA,
                VICTORY_POSE_REWARD_PREFIX,
                "VictoryPoseSchematicItem::GrantReward",
            ),
        }
    }
}

struct SchematicRewardTarget {
    schematic: usize,
    item: usize,
    kind: SchematicRewardKind,
    overclock_id: Option<[u8; 16]>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClassLevelChange {
    pub class_name: String,
    pub previous_xp: i32,
    pub current_xp: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaxClassLevelResult {
    pub pid: u32,
    pub classes_updated: usize,
    pub changes: Vec<ClassLevelChange>,
    pub backup_path: String,
    pub message: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionChange {
    pub class_name: String,
    pub previous_promotions: i32,
    pub current_promotions: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PromoteAllResult {
    pub pid: u32,
    pub classes_promoted: usize,
    pub changes: Vec<PromotionChange>,
    pub backup_path: String,
    pub message: String,
}

struct CharacterProgress {
    class_name: &'static str,
    xp_address: usize,
    xp: i32,
    promotions_address: usize,
    promotions: i32,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipStatus {
    pub enabled: bool,
    pub available: bool,
    pub pid: Option<u32>,
    pub weapon_name: Option<String>,
    pub clip_count: Option<i32>,
    pub clip_size: Option<i32>,
    pub address: Option<String>,
    pub message: String,
}

impl ClipStatus {
    fn pending(enabled: bool, message: impl Into<String>) -> Self {
        Self {
            enabled,
            available: false,
            pid: None,
            weapon_name: None,
            clip_count: None,
            clip_size: None,
            address: None,
            message: message.into(),
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DamageStatus {
    pub enabled: bool,
    pub available: bool,
    pub pid: Option<u32>,
    pub weapon_name: Option<String>,
    pub value: Option<f32>,
    pub target_count: usize,
    pub addresses: Vec<String>,
    pub message: String,
}

impl DamageStatus {
    fn pending(enabled: bool, message: impl Into<String>) -> Self {
        Self {
            enabled,
            available: false,
            pid: None,
            weapon_name: None,
            value: None,
            target_count: 0,
            addresses: Vec::new(),
            message: message.into(),
        }
    }
}

mod process;
use process::{OwnedHandle, ProcessMemory};

mod save_reader;
use save_reader::CreditsReader;

mod unreal_runtime;
use unreal_runtime::ClipReader;

fn wide_string(buffer: &[u16]) -> String {
    let length = buffer
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..length])
}

fn find_process_id() -> Result<u32> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
        .map_err(|error| MemoryError(format!("Falha ao listar processos: {error}")))?;
    let snapshot = OwnedHandle(snapshot);
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut available = unsafe { Process32FirstW(snapshot.raw(), &mut entry) }.is_ok();
    while available {
        if wide_string(&entry.szExeFile).eq_ignore_ascii_case(PROCESS_NAME) {
            return Ok(entry.th32ProcessID);
        }
        available = unsafe { Process32NextW(snapshot.raw(), &mut entry) }.is_ok();
    }
    Err(MemoryError(format!("Aguardando {PROCESS_NAME}.")))
}

fn find_main_module(pid: u32) -> Result<(usize, PathBuf)> {
    let snapshot =
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) }
            .map_err(|error| MemoryError(format!("Falha ao listar modulos: {error}")))?;
    let snapshot = OwnedHandle(snapshot);
    let mut entry = MODULEENTRY32W {
        dwSize: size_of::<MODULEENTRY32W>() as u32,
        ..Default::default()
    };
    let mut available = unsafe { Module32FirstW(snapshot.raw(), &mut entry) }.is_ok();
    while available {
        if wide_string(&entry.szModule).eq_ignore_ascii_case(PROCESS_NAME) {
            let base = entry.modBaseAddr as usize;
            return Ok((base, PathBuf::from(wide_string(&entry.szExePath))));
        }
        available = unsafe { Module32NextW(snapshot.raw(), &mut entry) }.is_ok();
    }
    Err(MemoryError(format!(
        "Modulo {PROCESS_NAME} nao encontrado."
    )))
}

fn validate_build(path: &Path) -> Result<()> {
    let metadata = std::fs::metadata(path)?;
    let file_size = metadata.len();
    let modified = metadata
        .modified()?
        .duration_since(UNIX_EPOCH)
        .map_err(|error| MemoryError(error.to_string()))?
        .as_nanos();
    let cache = BUILD_CACHE.get_or_init(|| Mutex::new(None));
    {
        let guard = cache
            .lock()
            .map_err(|_| MemoryError("Cache de build indisponivel.".into()))?;
        if guard
            .as_ref()
            .is_some_and(|(cached_path, cached_size, cached_modified)| {
                cached_path == path && *cached_size == file_size && *cached_modified == modified
            })
        {
            return Ok(());
        }
    }

    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let actual = format!("{:x}", hasher.finalize());
    if actual != EXPECTED_EXE_SHA256 {
        return Err(MemoryError(format!(
            "A build do jogo mudou. SHA-256 atual: {actual}. Os offsets foram bloqueados."
        )));
    }
    *cache
        .lock()
        .map_err(|_| MemoryError("Cache de build indisponivel.".into()))? =
        Some((path.to_path_buf(), file_size, modified));
    Ok(())
}

fn backup_save_games(executable: &Path, operation: &str) -> Result<PathBuf> {
    let fsd_root = executable
        .ancestors()
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("FSD"))
        })
        .ok_or_else(|| {
            MemoryError("Diretorio FSD nao encontrado a partir do executavel.".into())
        })?;
    let save_dir = fsd_root.join("Saved").join("SaveGames");
    if !save_dir.is_dir() {
        return Err(MemoryError(format!(
            "Diretorio de saves nao encontrado: {}",
            save_dir.display()
        )));
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| MemoryError(format!("Relogio do sistema invalido: {error}")))?
        .as_secs();
    let backup_dir = save_dir
        .join("DRGTrainerBackups")
        .join(format!("{operation}-{timestamp}"));
    fs::create_dir_all(&backup_dir).map_err(|error| {
        MemoryError(format!(
            "Nao foi possivel criar o backup em {}: {error}",
            backup_dir.display()
        ))
    })?;

    let mut copied = 0_usize;
    for entry in fs::read_dir(&save_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let name_text = name.to_string_lossy();
        let main_save = name_text.ends_with("_Player.sav");
        let slot_save = name_text.contains("_Player_Slot_")
            && !name_text.contains("_ExternalBackup_")
            && name_text.ends_with(".sav");
        if !main_save && !slot_save {
            continue;
        }
        fs::copy(entry.path(), backup_dir.join(&name)).map_err(|error| {
            MemoryError(format!(
                "Falha ao copiar {name_text} para o backup: {error}"
            ))
        })?;
        copied += 1;
    }
    if copied == 0 {
        let _ = fs::remove_dir_all(&backup_dir);
        return Err(MemoryError(
            "Nenhum save principal foi encontrado para criar o backup.".into(),
        ));
    }
    Ok(backup_dir)
}

fn context(writable: bool) -> Result<(u32, CreditsReader)> {
    let pid = find_process_id()?;
    let (module_base, executable) = find_main_module(pid)?;
    validate_build(&executable)?;
    Ok((pid, CreditsReader::connect(pid, module_base, writable)?))
}

fn clip_context(writable: bool) -> Result<ClipReader> {
    let pid = find_process_id()?;
    let (module_base, executable) = find_main_module(pid)?;
    validate_build(&executable)?;
    ClipReader::connect(pid, module_base, writable)
}

fn clip_status_store() -> &'static Mutex<ClipStatus> {
    CLIP_STATUS.get_or_init(|| Mutex::new(ClipStatus::pending(false, "Desativado.")))
}

fn store_clip_status(status: ClipStatus) {
    if let Ok(mut current) = clip_status_store().lock() {
        *current = status;
    }
}

fn damage_status_store() -> &'static Mutex<DamageStatus> {
    DAMAGE_STATUS.get_or_init(|| Mutex::new(DamageStatus::pending(false, "Desativado.")))
}

fn store_damage_status(status: DamageStatus) {
    if let Ok(mut current) = damage_status_store().lock() {
        *current = status;
    }
}

fn start_clip_worker() {
    if CLIP_WORKER_RUNNING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }

    thread::spawn(|| {
        while INFINITE_MAGAZINE_ENABLED.load(Ordering::Acquire) {
            match clip_context(true) {
                Ok(reader) => {
                    while INFINITE_MAGAZINE_ENABLED.load(Ordering::Acquire) {
                        match reader.snapshot(true) {
                            Ok(status) => store_clip_status(status),
                            Err(error) => {
                                store_clip_status(ClipStatus::pending(true, error.to_string()));
                                break;
                            }
                        }
                        thread::sleep(FREEZE_INTERVAL);
                    }
                }
                Err(error) => {
                    store_clip_status(ClipStatus::pending(true, error.to_string()));
                }
            }
            if INFINITE_MAGAZINE_ENABLED.load(Ordering::Acquire) {
                thread::sleep(RECONNECT_INTERVAL);
            }
        }

        if let Ok(mut status) = clip_status_store().lock() {
            status.enabled = false;
            status.message = "Infinite Magazine desativado.".into();
        }
        CLIP_WORKER_RUNNING.store(false, Ordering::Release);

        // Cobre o caso raro de reativacao enquanto o worker anterior encerrava.
        if INFINITE_MAGAZINE_ENABLED.load(Ordering::Acquire) {
            start_clip_worker();
        }
    });
}

fn start_damage_worker() {
    if DAMAGE_WORKER_RUNNING
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return;
    }

    thread::spawn(|| {
        while WEAPON_DAMAGE_ENABLED.load(Ordering::Acquire) {
            match clip_context(true) {
                Ok(reader) => {
                    while WEAPON_DAMAGE_ENABLED.load(Ordering::Acquire) {
                        match reader.damage_snapshot(true) {
                            Ok(status) => store_damage_status(status),
                            Err(error) => {
                                store_damage_status(DamageStatus::pending(true, error.to_string()));
                                break;
                            }
                        }
                        thread::sleep(FREEZE_INTERVAL);
                    }
                }
                Err(error) => {
                    store_damage_status(DamageStatus::pending(true, error.to_string()));
                }
            }
            if WEAPON_DAMAGE_ENABLED.load(Ordering::Acquire) {
                thread::sleep(RECONNECT_INTERVAL);
            }
        }

        if let Ok(mut status) = damage_status_store().lock() {
            status.enabled = false;
            status.message = "Weapon Damage desativado.".into();
        }
        DAMAGE_WORKER_RUNNING.store(false, Ordering::Release);

        if WEAPON_DAMAGE_ENABLED.load(Ordering::Acquire) {
            start_damage_worker();
        }
    });
}

pub fn read_credits() -> Result<CreditSnapshot> {
    let (pid, reader) = context(false)?;
    let (credits, address) = reader.read()?;
    Ok(CreditSnapshot {
        credits,
        pid,
        address: format!("0x{address:X}"),
        module_base: format!("0x{:X}", reader.module_base),
        offset: format!("0x{CREDITS_OFFSET:X}"),
        build_valid: true,
    })
}

pub fn set_credits(value: i32) -> Result<WriteResult> {
    let (pid, reader) = context(true)?;
    let (previous, current, address) = reader.write(value)?;
    Ok(WriteResult {
        previous,
        current,
        pid,
        address: format!("0x{address:X}"),
    })
}

mod resource_catalog;
mod resources;

use resource_catalog::{RESOURCE_DEFINITIONS, ResourceDefinition};
pub use resources::{add_all_resources, add_resource, read_resources};

mod progression;
pub use progression::{
    max_class_level, promote_all_classes, unlock_all_gear_modifications,
    unlock_all_overclocks_and_cosmetics, unlock_all_perks, unlock_all_weapons,
};

mod weapons;
pub use weapons::{
    infinite_magazine_enabled, read_clip_status, read_damage_status, set_infinite_magazine,
    set_weapon_damage, weapon_damage_enabled,
};

#[cfg(test)]
mod tests;
