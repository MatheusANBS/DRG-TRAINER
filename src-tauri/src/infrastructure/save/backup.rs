//! Backup versionado de saves (SPEC-020).
//!
//! Localiza o diretorio de saves a partir do executavel e copia os arquivos
//! relevantes para um diretorio novo por operacao. Um backup nunca sobrescreve
//! outro: se o nome ja existir, um sufixo e adicionado.
//!
//! Politica de retencao: backups **nunca** sao removidos automaticamente. Eles
//! sao a rede de seguranca do usuario, e apagar automaticamente trocaria espaco
//! em disco por risco de perda de progresso. `BackupSet::total_backups` informa
//! quantos existem para que a UI possa sugerir uma limpeza manual.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::shared::error::{ErrorCode, Result, TrainerError};

/// Nome do diretorio raiz onde os backups do trainer ficam.
pub const BACKUP_ROOT: &str = "DRGTrainerBackups";

/// Resultado de um backup concluido.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSet {
    /// Diretorio criado para esta operacao.
    pub path: String,
    /// Quantidade de arquivos de save copiados.
    pub files: usize,
    /// Quantos backups o trainer ja acumulou nesta instalacao.
    pub total_backups: usize,
}

impl BackupSet {
    pub fn path(&self) -> &Path {
        Path::new(&self.path)
    }
}

fn backup_error(message: impl Into<String>) -> TrainerError {
    TrainerError::new(ErrorCode::BackupFailed, message)
}

/// Localiza `FSD/Saved/SaveGames` a partir do caminho do executavel.
pub fn locate_save_directory(executable: &Path) -> Result<PathBuf> {
    let fsd_root = executable
        .ancestors()
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("FSD"))
        })
        .ok_or_else(|| backup_error("Diretorio FSD nao encontrado a partir do executavel."))?;

    let save_dir = fsd_root.join("Saved").join("SaveGames");
    if !save_dir.is_dir() {
        return Err(backup_error(format!(
            "Diretorio de saves nao encontrado: {}",
            save_dir.display()
        )));
    }
    Ok(save_dir)
}

/// Um arquivo entra no backup se for o save principal ou um slot legitimo.
/// Backups externos do proprio jogo sao ignorados para nao duplicar dados.
fn is_backup_candidate(file_name: &str) -> bool {
    let main_save = file_name.ends_with("_Player.sav");
    let slot_save = file_name.contains("_Player_Slot_")
        && !file_name.contains("_ExternalBackup_")
        && file_name.ends_with(".sav");
    main_save || slot_save
}

/// Escolhe um diretorio de destino que ainda nao exista.
///
/// A granularidade do timestamp e de um segundo; duas operacoes seguidas
/// receberiam o mesmo nome e a segunda sobrescreveria a primeira em silencio.
fn unique_backup_dir(root: &Path, operation: &str, timestamp: u64) -> Result<PathBuf> {
    let base = root.join(format!("{operation}-{timestamp}"));
    if !base.exists() {
        return Ok(base);
    }
    for attempt in 2..1_000 {
        let candidate = root.join(format!("{operation}-{timestamp}-{attempt}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(backup_error(
        "Nao foi possivel criar um diretorio de backup exclusivo.",
    ))
}

fn count_existing_backups(root: &Path) -> usize {
    fs::read_dir(root)
        .map(|entries| {
            entries
                .filter_map(std::result::Result::ok)
                .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
                .count()
        })
        .unwrap_or(0)
}

/// Copia os saves de `save_dir` para um diretorio novo sob `BACKUP_ROOT`.
pub fn create_backup_in(save_dir: &Path, operation: &str) -> Result<BackupSet> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| backup_error(format!("Relogio do sistema invalido: {error}")))?
        .as_secs();

    let root = save_dir.join(BACKUP_ROOT);
    fs::create_dir_all(&root).map_err(|error| {
        backup_error(format!(
            "Nao foi possivel criar {}: {error}",
            root.display()
        ))
    })?;
    let backup_dir = unique_backup_dir(&root, operation, timestamp)?;
    fs::create_dir(&backup_dir).map_err(|error| {
        backup_error(format!(
            "Nao foi possivel criar o backup em {}: {error}",
            backup_dir.display()
        ))
    })?;

    let mut copied = 0_usize;
    for entry in fs::read_dir(save_dir)
        .map_err(|error| backup_error(format!("Falha ao listar os saves: {error}")))?
    {
        let entry =
            entry.map_err(|error| backup_error(format!("Falha ao listar os saves: {error}")))?;
        if !entry
            .file_type()
            .map_err(|error| backup_error(error.to_string()))?
            .is_file()
        {
            continue;
        }
        let name = entry.file_name();
        let name_text = name.to_string_lossy();
        if !is_backup_candidate(&name_text) {
            continue;
        }
        fs::copy(entry.path(), backup_dir.join(&name)).map_err(|error| {
            backup_error(format!(
                "Falha ao copiar {name_text} para o backup: {error}"
            ))
        })?;
        copied += 1;
    }

    if copied == 0 {
        // Um diretorio de backup vazio daria falsa sensacao de seguranca.
        let _ = fs::remove_dir(&backup_dir);
        return Err(backup_error(
            "Nenhum save principal foi encontrado para criar o backup.",
        ));
    }

    Ok(BackupSet {
        path: backup_dir.display().to_string(),
        files: copied,
        total_backups: count_existing_backups(&root),
    })
}

/// Cria o backup resolvendo o diretorio de saves pelo executavel.
pub fn create_backup(executable: &Path, operation: &str) -> Result<BackupSet> {
    create_backup_in(&locate_save_directory(executable)?, operation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let mut path = std::env::temp_dir();
            path.push(format!(
                "drg-backup-{label}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("diretorio temporario");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }

        fn write(&self, name: &str, contents: &str) {
            let mut file = fs::File::create(self.0.join(name)).expect("arquivo de save");
            file.write_all(contents.as_bytes()).expect("escrita");
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn copies_main_and_slot_saves_only() {
        let dir = TempDir::new("selecao");
        dir.write("76561198000000000_Player.sav", "principal");
        dir.write("76561198000000000_Player_Slot_1.sav", "slot");
        dir.write(
            "76561198000000000_Player_Slot_ExternalBackup_1.sav",
            "externo",
        );
        dir.write("Config.ini", "irrelevante");

        let backup = create_backup_in(dir.path(), "unlock-all").expect("backup deve ser criado");
        assert_eq!(backup.files, 2);

        let copied = fs::read_dir(backup.path())
            .expect("backup deve existir")
            .filter_map(std::result::Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert!(copied.iter().any(|name| name.ends_with("_Player.sav")));
        assert!(copied.iter().any(|name| name.contains("_Player_Slot_1")));
        assert!(!copied.iter().any(|name| name.contains("ExternalBackup")));
        assert!(!copied.iter().any(|name| name.ends_with(".ini")));
    }

    #[test]
    fn a_second_backup_in_the_same_second_never_overwrites_the_first() {
        let dir = TempDir::new("colisao");
        dir.write("player_Player.sav", "v1");

        let first = create_backup_in(dir.path(), "promote").expect("primeiro backup");
        dir.write("player_Player.sav", "v2");
        let second = create_backup_in(dir.path(), "promote").expect("segundo backup");

        assert_ne!(first.path, second.path, "os backups devem ser distintos");
        let original = fs::read_to_string(first.path().join("player_Player.sav")).unwrap();
        let updated = fs::read_to_string(second.path().join("player_Player.sav")).unwrap();
        assert_eq!(original, "v1", "o backup anterior deve permanecer intacto");
        assert_eq!(updated, "v2");
        assert_eq!(second.total_backups, 2);
    }

    #[test]
    fn refuses_to_report_success_when_there_is_nothing_to_back_up() {
        let dir = TempDir::new("vazio");
        dir.write("Config.ini", "irrelevante");

        let error =
            create_backup_in(dir.path(), "unlock-all").expect_err("sem saves o backup deve falhar");
        assert_eq!(error.code(), ErrorCode::BackupFailed);

        // E nao deve deixar um diretorio vazio para tras.
        let root = dir.path().join(BACKUP_ROOT);
        assert_eq!(count_existing_backups(&root), 0);
    }

    #[test]
    fn locates_the_save_directory_from_the_executable_path() {
        let dir = TempDir::new("layout");
        let executable = dir
            .path()
            .join("FSD")
            .join("Binaries")
            .join("Win64")
            .join("FSD-Win64-Shipping.exe");
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        let saves = dir.path().join("FSD").join("Saved").join("SaveGames");
        fs::create_dir_all(&saves).unwrap();

        assert_eq!(locate_save_directory(&executable).unwrap(), saves);
    }

    #[test]
    fn missing_save_directory_is_reported_as_backup_failure() {
        let dir = TempDir::new("sem-saves");
        let executable = dir.path().join("FSD").join("FSD-Win64-Shipping.exe");
        fs::create_dir_all(executable.parent().unwrap()).unwrap();

        let error = locate_save_directory(&executable).expect_err("sem SaveGames deve falhar");
        assert_eq!(error.code(), ErrorCode::BackupFailed);
    }

    #[test]
    fn executable_outside_an_fsd_tree_is_rejected() {
        let dir = TempDir::new("fora");
        let executable = dir.path().join("Jogo").join("FSD-Win64-Shipping.exe");
        fs::create_dir_all(executable.parent().unwrap()).unwrap();

        assert_eq!(
            locate_save_directory(&executable)
                .expect_err("caminho sem FSD deve falhar")
                .code(),
            ErrorCode::BackupFailed
        );
    }

    #[test]
    fn file_selection_matches_the_documented_rule() {
        assert!(is_backup_candidate("123_Player.sav"));
        assert!(is_backup_candidate("123_Player_Slot_2.sav"));
        assert!(!is_backup_candidate("123_Player_Slot_ExternalBackup_2.sav"));
        assert!(!is_backup_candidate("123_Player.sav.bak"));
        assert!(!is_backup_candidate("Settings.ini"));
    }
}
