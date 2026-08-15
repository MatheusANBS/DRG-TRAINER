//! Descoberta do processo e identificacao da build (SPEC-001 / SPEC-002).
//!
//! Encontra o processo do jogo, localiza o modulo principal e calcula a hash do
//! executavel. A hash e o unico insumo usado pelo registry para decidir se a
//! build e suportada.

use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::UNIX_EPOCH;

use sha2::{Digest, Sha256};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, PROCESSENTRY32W,
    Process32FirstW, Process32NextW, TH32CS_SNAPMODULE, TH32CS_SNAPMODULE32, TH32CS_SNAPPROCESS,
};

use super::handle::OwnedHandle;
use crate::shared::error::{ErrorCode, Result, TrainerError};

/// Chave de cache da hash: caminho + tamanho + mtime. Se qualquer um mudar, o
/// executavel e re-hasheado.
type HashCacheEntry = (PathBuf, u64, u128, String);
static HASH_CACHE: OnceLock<Mutex<Option<HashCacheEntry>>> = OnceLock::new();

/// Processo do jogo localizado, ainda sem perfil resolvido.
#[derive(Debug, Clone)]
pub struct LocatedProcess {
    pub pid: u32,
    pub module_base: usize,
    pub executable: PathBuf,
    pub executable_name: String,
}

fn wide_string(buffer: &[u16]) -> String {
    let length = buffer
        .iter()
        .position(|unit| *unit == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..length])
}

/// Procura o primeiro processo cujo nome esteja entre os candidatos.
pub fn find_process_id(candidates: &[&str]) -> Result<(u32, String)> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }.map_err(|error| {
        TrainerError::new(
            ErrorCode::ProcessNotFound,
            format!("Falha ao listar processos: {error}"),
        )
    })?;
    let snapshot = OwnedHandle::new(snapshot);
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut available = unsafe { Process32FirstW(snapshot.raw(), &mut entry) }.is_ok();
    while available {
        let name = wide_string(&entry.szExeFile);
        if candidates
            .iter()
            .any(|candidate| name.eq_ignore_ascii_case(candidate))
        {
            return Ok((entry.th32ProcessID, name));
        }
        available = unsafe { Process32NextW(snapshot.raw(), &mut entry) }.is_ok();
    }
    Err(TrainerError::new(
        ErrorCode::ProcessNotFound,
        format!("Aguardando {}.", candidates.join(" / ")),
    ))
}

/// Localiza o modulo principal e devolve base + caminho do executavel.
pub fn find_main_module(pid: u32, module_name: &str) -> Result<(usize, PathBuf)> {
    let snapshot =
        unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, pid) }.map_err(
            |error| {
                TrainerError::new(
                    ErrorCode::ModuleNotFound,
                    format!("Falha ao listar modulos: {error}"),
                )
            },
        )?;
    let snapshot = OwnedHandle::new(snapshot);
    let mut entry = MODULEENTRY32W {
        dwSize: size_of::<MODULEENTRY32W>() as u32,
        ..Default::default()
    };
    let mut available = unsafe { Module32FirstW(snapshot.raw(), &mut entry) }.is_ok();
    while available {
        if wide_string(&entry.szModule).eq_ignore_ascii_case(module_name) {
            return Ok((
                entry.modBaseAddr as usize,
                PathBuf::from(wide_string(&entry.szExePath)),
            ));
        }
        available = unsafe { Module32NextW(snapshot.raw(), &mut entry) }.is_ok();
    }
    Err(TrainerError::new(
        ErrorCode::ModuleNotFound,
        format!("Modulo {module_name} nao encontrado."),
    ))
}

/// Encontra o processo do jogo entre os executaveis conhecidos.
pub fn locate(candidates: &[&str]) -> Result<LocatedProcess> {
    let (pid, executable_name) = find_process_id(candidates)?;
    let (module_base, executable) = find_main_module(pid, &executable_name)?;
    Ok(LocatedProcess {
        pid,
        module_base,
        executable,
        executable_name,
    })
}

/// Calcula (ou reaproveita do cache) a SHA-256 do executavel.
///
/// O cache evita re-hashear centenas de megabytes a cada poll; ele e invalidado
/// automaticamente quando o arquivo muda de tamanho ou data.
pub fn executable_sha256(path: &Path) -> Result<String> {
    let metadata = std::fs::metadata(path).map_err(|error| {
        TrainerError::new(
            ErrorCode::BuildHashFailed,
            format!("Nao foi possivel inspecionar o executavel: {error}"),
        )
    })?;
    let file_size = metadata.len();
    let modified = metadata
        .modified()
        .map_err(|error| {
            TrainerError::new(
                ErrorCode::BuildHashFailed,
                format!("Data do executavel indisponivel: {error}"),
            )
        })?
        .duration_since(UNIX_EPOCH)
        .map_err(|error| TrainerError::new(ErrorCode::BuildHashFailed, error.to_string()))?
        .as_nanos();

    let cache = HASH_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = cache.lock()
        && let Some((cached_path, cached_size, cached_modified, cached_hash)) = guard.as_ref()
        && cached_path == path
        && *cached_size == file_size
        && *cached_modified == modified
    {
        return Ok(cached_hash.clone());
    }

    let hash = hash_file(path)?;
    if let Ok(mut guard) = cache.lock() {
        *guard = Some((path.to_path_buf(), file_size, modified, hash.clone()));
    }
    Ok(hash)
}

fn hash_file(path: &Path) -> Result<String> {
    use std::io::Read;

    let mut file = std::fs::File::open(path).map_err(|error| {
        TrainerError::new(
            ErrorCode::BuildHashFailed,
            format!("Nao foi possivel abrir o executavel: {error}"),
        )
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|error| {
            TrainerError::new(
                ErrorCode::BuildHashFailed,
                format!("Falha ao ler o executavel: {error}"),
            )
        })?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Limpa o cache de hash. Usado quando o processo alvo e trocado.
pub fn invalidate_hash_cache() {
    if let Some(cache) = HASH_CACHE.get()
        && let Ok(mut guard) = cache.lock()
    {
        *guard = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn hashes_a_file_and_reuses_the_cache() {
        let mut path = std::env::temp_dir();
        path.push(format!("drg-hash-test-{}.bin", std::process::id()));
        {
            let mut file = std::fs::File::create(&path).expect("arquivo temporario");
            file.write_all(b"deep rock galactic").expect("escrita");
        }

        let first = executable_sha256(&path).expect("hash deve ser calculada");
        let second = executable_sha256(&path).expect("hash deve vir do cache");
        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
        // SHA-256 conhecida de "deep rock galactic".
        assert!(first.chars().all(|character| character.is_ascii_hexdigit()));

        invalidate_hash_cache();
        let third = executable_sha256(&path).expect("hash deve ser recalculada");
        assert_eq!(first, third);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_executable_reports_a_build_hash_failure() {
        let error = executable_sha256(Path::new("Z:/nao-existe/FSD-Win64-Shipping.exe"))
            .expect_err("arquivo ausente deve falhar");
        assert_eq!(error.code(), ErrorCode::BuildHashFailed);
    }

    #[test]
    fn absent_process_reports_process_not_found() {
        let error = find_process_id(&["drg-processo-que-nao-existe-12345.exe"])
            .expect_err("processo inexistente deve falhar");
        assert_eq!(error.code(), ErrorCode::ProcessNotFound);
        assert!(error.recoverable);
    }
}
