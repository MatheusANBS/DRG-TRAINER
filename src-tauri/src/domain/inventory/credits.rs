//! Creditos do save ativo.
//!
//! Mutacao de runtime: escreve no save carregado em memoria e verifica a
//! releitura. Nao grava em disco e, por isso, nao abre transacao de backup —
//! a persistencia continua sendo decisao do jogo (SPEC-015: acao `standard`).

use crate::build_profiles::Capability;
use crate::domain::dto::{CreditSnapshot, WriteResult};
use crate::domain::save_game::SaveGame;
use crate::domain::session::TrainerSession;
use crate::infrastructure::process::MemoryWriter;
use crate::shared::error::{Result, codes};

pub fn read_credits() -> Result<CreditSnapshot> {
    let session = TrainerSession::read_only()?;
    session.require(Capability::Credits)?;

    let save = SaveGame::resolve(session.runtime())?;
    let (credits, address) = save.read_credits()?;

    Ok(CreditSnapshot {
        credits,
        pid: session.pid(),
        address: format!("0x{address:X}"),
        module_base: format!("0x{:X}", session.runtime().module_base()),
        offset: format!("0x{:X}", session.profile().offsets.save.credits),
    })
}

pub fn set_credits(value: i32) -> Result<WriteResult> {
    // O teto e o proprio `i32` do campo no save; so o piso precisa de guarda.
    if value < 0 {
        return Err(codes::invalid_argument("O saldo nao pode ser negativo."));
    }

    let session = TrainerSession::writable()?;
    session.require(Capability::Credits)?;

    let save = SaveGame::resolve(session.runtime())?;
    let address = save.credits_address()?;
    // A escrita so e reportada como sucesso depois de relida e conferida.
    let previous = session
        .runtime()
        .memory()
        .write_i32_verified(address, value)?;

    Ok(WriteResult {
        previous,
        current: value,
        pid: session.pid(),
        address: format!("0x{address:X}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::ErrorCode;

    #[test]
    fn negative_balances_are_refused_before_touching_the_process() {
        let error = set_credits(-1).expect_err("saldo negativo deve ser recusado");
        assert_eq!(error.code(), ErrorCode::InvalidArgument);
    }
}
