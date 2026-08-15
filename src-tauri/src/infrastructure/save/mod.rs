//! Servico unico de mutacao persistente de save (SPEC-020).
//!
//! Contrato: nenhum modulo de dominio escreve no save sem abrir uma
//! `SaveTransaction`. O pipeline e sempre o mesmo:
//!
//! 1. localizar o diretorio de saves a partir do executavel;
//! 2. validar que ha o que preservar;
//! 3. criar um backup versionado (nunca sobrescrito);
//! 4. executar a mutacao;
//! 5. reler e verificar o resultado;
//! 6. reportar, incluindo o caminho de restauracao quando algo falhar.
//!
//! Depois do passo 3, qualquer erro passa por `guard`/`verify`, de modo que a
//! mensagem entregue ao usuario sempre diz onde estao os artefatos.

pub mod backup;

use std::path::Path;

pub use backup::{BACKUP_ROOT, BackupSet, create_backup, create_backup_in, locate_save_directory};

use crate::shared::error::{ErrorCode, Result, TrainerError};

/// Transacao de mutacao permanente com backup ja garantido.
#[derive(Debug)]
pub struct SaveTransaction {
    backup: BackupSet,
}

impl SaveTransaction {
    /// Executa os passos 1 a 3. Se qualquer um falhar, nenhuma mutacao ocorre.
    pub fn begin(executable: &Path, operation: &str) -> Result<Self> {
        Ok(Self {
            backup: create_backup(executable, operation)?,
        })
    }

    /// Variante testavel que recebe o diretorio de saves diretamente.
    pub fn begin_in(save_dir: &Path, operation: &str) -> Result<Self> {
        Ok(Self {
            backup: create_backup_in(save_dir, operation)?,
        })
    }

    pub fn backup(&self) -> &BackupSet {
        &self.backup
    }

    pub fn backup_path(&self) -> &str {
        &self.backup.path
    }

    /// Anexa o caminho de restauracao a um erro pos-backup, preservando o
    /// codigo original para o frontend.
    pub fn wrap(&self, error: TrainerError) -> TrainerError {
        TrainerError::new(
            error.code(),
            format!(
                "{}. O backup permanece em {}.",
                error.message().trim_end_matches('.'),
                self.backup.path
            ),
        )
    }

    /// Versao de `wrap` para resultados.
    pub fn guard<T>(&self, result: Result<T>) -> Result<T> {
        result.map_err(|error| self.wrap(error))
    }

    /// Falha de verificacao pos-escrita: o dado nao ficou como esperado.
    pub fn verify(&self, condition: bool, message: impl Into<String>) -> Result<()> {
        if condition {
            return Ok(());
        }
        Err(TrainerError::new(
            ErrorCode::WriteVerificationFailed,
            format!(
                "{}. O backup permanece em {}.",
                message.into().trim_end_matches('.'),
                self.backup.path
            ),
        ))
    }

    /// O save ativo mudou de identidade no meio da operacao.
    pub fn ensure_same_save(&self, expected: usize, actual: usize) -> Result<()> {
        if expected == actual {
            return Ok(());
        }
        Err(TrainerError::new(
            ErrorCode::SaveStateChanged,
            format!(
                "O save ativo mudou durante a operacao. O backup permanece em {}.",
                self.backup.path
            ),
        ))
    }

    /// Encerra a transacao devolvendo o backup para o resultado da operacao.
    pub fn commit(self) -> BackupSet {
        self.backup
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    struct TempSaves(PathBuf);

    impl TempSaves {
        fn new(label: &str) -> Self {
            let mut path = std::env::temp_dir();
            path.push(format!("drg-tx-{label}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&path);
            fs::create_dir_all(&path).expect("diretorio temporario");
            let mut file = fs::File::create(path.join("player_Player.sav")).unwrap();
            file.write_all(b"save").unwrap();
            Self(path)
        }
    }

    impl Drop for TempSaves {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn a_transaction_always_produces_a_backup_before_any_mutation() {
        let saves = TempSaves::new("ordem");
        let transaction = SaveTransaction::begin_in(&saves.0, "max-class-level").unwrap();
        assert!(Path::new(transaction.backup_path()).is_dir());
        assert_eq!(transaction.backup().files, 1);
    }

    #[test]
    fn guard_preserves_the_error_code_and_points_at_the_backup() {
        let saves = TempSaves::new("guard");
        let transaction = SaveTransaction::begin_in(&saves.0, "promote").unwrap();

        let failure: Result<()> = Err(TrainerError::new(
            ErrorCode::NativeCallFailed,
            "A rotina nao retornou.",
        ));
        let error = transaction
            .guard(failure)
            .expect_err("erro deve ser propagado");
        assert_eq!(error.code(), ErrorCode::NativeCallFailed);
        assert!(error.message().contains(transaction.backup_path()));
    }

    #[test]
    fn verify_reports_a_write_verification_failure() {
        let saves = TempSaves::new("verify");
        let transaction = SaveTransaction::begin_in(&saves.0, "unlock-all").unwrap();

        assert!(transaction.verify(true, "tudo certo").is_ok());
        let error = transaction
            .verify(false, "Os contadores nao mudaram")
            .expect_err("verificacao falha deve virar erro");
        assert_eq!(error.code(), ErrorCode::WriteVerificationFailed);
        assert!(error.message().contains("backup permanece em"));
    }

    #[test]
    fn a_changed_active_save_aborts_the_transaction() {
        let saves = TempSaves::new("troca");
        let transaction = SaveTransaction::begin_in(&saves.0, "unlock-all").unwrap();

        assert!(transaction.ensure_same_save(0x1000, 0x1000).is_ok());
        let error = transaction
            .ensure_same_save(0x1000, 0x2000)
            .expect_err("save diferente deve abortar");
        assert_eq!(error.code(), ErrorCode::SaveStateChanged);
    }

    #[test]
    fn a_failed_backup_prevents_the_transaction_from_starting() {
        let mut empty = std::env::temp_dir();
        empty.push(format!("drg-tx-vazio-{}", std::process::id()));
        let _ = fs::remove_dir_all(&empty);
        fs::create_dir_all(&empty).unwrap();

        let error = SaveTransaction::begin_in(&empty, "unlock-all")
            .expect_err("sem saves a transacao nao pode comecar");
        assert_eq!(error.code(), ErrorCode::BackupFailed);

        let _ = fs::remove_dir_all(&empty);
    }
}
