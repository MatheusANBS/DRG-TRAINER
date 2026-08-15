# Scorecard de qualidade

Transforma "está melhor" em algo verificável (SPEC-022). A avaliação de origem
era ~8,1/10 geral e ~8,3/10 visual; o alvo é 9+ sem perder os pontos fortes.

Um critério só conta como atendido quando existe **evidência automatizada** ou
um item correspondente no [checklist de release](release-checklist.md).

## Definition of Done do marco 1.0

| # | Critério | Como se verifica | Status |
| --- | --- | --- | --- |
| 1 | CI verde em todos os gates | Jobs `frontend`, `rust-format`, `rust-lint`, `rust-test`, `dependency-audit`, `codeql` | ✅ |
| 2 | Nenhum warning de Clippy | `cargo clippy --locked --all-targets -- -D warnings` | ✅ |
| 3 | Testes offline independentes do jogo | `cargo test` e `npm test` em máquina sem o DRG | ✅ |
| 4 | Checklist de live tests aprovado | [release-checklist.md](release-checklist.md), seção de regressão | Por release |
| 5 | CSP ativa | `src-tauri/tauri.conf.json` sem `csp: null` | ✅ |
| 6 | Build profile modular | `src-tauri/src/build_profiles/` resolve por hash | ✅ |
| 7 | Frontend modularizado | `App.tsx` como composition root; `invoke` só em `services/` | ✅ |
| 8 | Release com checksums | `SHA256SUMS.txt` + `build-manifest.json` gerados pelo CI | ✅ |

## Dimensões

### Segurança de memória (SPEC-001)

| Critério | Evidência |
| --- | --- |
| Win32 confinado a uma camada | `infrastructure/process/` é o único módulo com `ReadProcessMemory`/`WriteProcessMemory`/`CreateRemoteThread` |
| Handles por RAII | `OwnedHandle` sem API de vazamento |
| Escrita valida bytes exatos | `write_bytes` compara `written` com o esperado |
| Falha nunca vira sucesso | `write_i32_verified` relê e confere |
| Ponteiro validado antes do dereference | `access::validate_range` |
| Nativa exige assinatura conferida | `call_verified` valida prefixo antes de chamar |

### Compatibilidade de build (SPEC-002 / SPEC-003)

| Critério | Evidência |
| --- | --- |
| Segunda build não exige editar lógica genérica | Perfil novo é um arquivo em `profiles/` + entrada em `ALL` |
| Build desconhecida bloqueia antes de operar | `TrainerSession::attach` recusa sem perfil `supported` |
| Frontend recebe perfil, hash e suporte | `get_trainer_status` → `BuildState` |
| Perfis têm testes de consistência | `build_profiles/registry.rs` |
| Procedimento reproduzível de onboarding | [build-support.md](build-support.md) |
| Capacidade não verificada fica desabilitada | `Capabilities` por perfil + `profile.require` |

### Qualidade e testes (SPEC-006 / SPEC-007 / SPEC-008)

| Critério | Evidência |
| --- | --- |
| `cargo test` roda sem o DRG instalado | Gate `rust-test` no CI |
| Live tests não rodam por padrão | Feature `live-tests` |
| Regras críticas com teste determinístico | `FakeMemory` + `TestWorld` + `SaveWorld` |
| Falhas de perfil/build com teste offline | `registry.rs`, `offline_contracts.rs` |
| Lint e teste do frontend no CI | Gate `frontend` |
| CodeQL para TS/Rust no CI | Gate `codeql` |
| Ações permanentes com teste de confirmação | `src/App.test.tsx` |
| Estados attached/unsupported/detached testados | `src/state/trainer-state.test.ts`, `src/App.test.tsx` |
| Mocks Tauri centralizados | `src/test/trainer-api-mock.ts` |

### Segurança da aplicação (SPEC-009 / SPEC-010 / SPEC-011)

| Critério | Evidência |
| --- | --- |
| CSP sem conteúdo remoto | `tauri.conf.json` |
| Capabilities mínimas e justificadas | `capabilities/default.json` + SECURITY.md |
| CI detecta advisories Rust | Gate `dependency-audit` |
| Exceções com justificativa e prazo | `src-tauri/.cargo/audit.toml` |
| Workflows com `permissions` explícitas | `.github/workflows/*.yml` |
| Actions pinadas por commit | Todas |
| Checksum gerado pelo CI | `release.yml` |
| Assinatura ligável sem redesenho | Stage condicional em `release.yml` |

### Save (SPEC-020)

| Critério | Evidência |
| --- | --- |
| Toda mutação permanente cria backup antes | `SaveTransaction::begin` precede a mutação |
| Backup nunca sobrescrito | `unique_backup_dir` + teste de colisão |
| Falha pós-escrita é detectada | `transaction.verify` / `ensure_same_save` |
| Instrução de restauração testada | [save-restore.md](save-restore.md) + checklist |

### Produto e interface (SPEC-013 a SPEC-019)

| Critério | Evidência |
| --- | --- |
| Sem polls concorrentes | `useTrainerPolling` agenda o próximo só ao fim do anterior |
| Unsupported build é estado de primeira classe | `TrainerConnection` |
| `disabled` derivado do estado central | `blockedReason` |
| Nenhum texto funcional abaixo de 9px | Tokens em `styles.css` |
| Focus ring visível | `:focus-visible` global |
| Toda ação tem semântica declarada | `src/types/actions.ts` + `ActionControl` |
| Mutação permanente distinguível antes do clique | Badge `PERMANENT` |
| Status legível em menos de um olhar | `TrainerStatusBar` |
| Sem overflow horizontal na largura mínima | Verificado em 760/820/821/940 |
| Nenhuma ação assíncrona dispara duplicada | Guard em `useAsyncAction` + teste |
| Erros conhecidos com mensagem específica | `src/lib/errors.ts` |

## Como atualizar

Revise a cada release, junto do checklist. Um critério marcado como atendido
sem evidência automatizada correspondente deve voltar para pendente — a tabela
só vale enquanto ninguém marcar por otimismo.
