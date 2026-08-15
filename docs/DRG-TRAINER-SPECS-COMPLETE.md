# DRG-TRAINER — Pacote Completo de Especificações

Este pacote transforma a avaliação técnica e visual do DRG-TRAINER em um backlog de especificações implementáveis.

## Escopo

Foram preservados tanto os pontos a melhorar quanto os pontos fortes que precisam virar invariantes, para que a evolução não cause regressão.

## SPECs

- **SPEC-001 — Invariantes de Segurança de Memória** — `P0` — Backend / Rust / Safety
- **SPEC-002 — BuildProfile e Compatibilidade Multi-Build** — `P0` — Arquitetura / Compatibilidade
- **SPEC-003 — Workflow de Atualização de Builds do DRG** — `P0` — Manutenção / Compatibilidade
- **SPEC-004 — Modularização do Backend Rust** — `P0` — Backend / Arquitetura
- **SPEC-005 — Decomposição do Unreal Runtime** — `P0` — Backend / Unreal
- **SPEC-006 — Arquitetura de Testes Offline e Live-Game** — `P0` — Qualidade / Testes
- **SPEC-007 — Quality Gates no CI/CD** — `P0` — CI/CD
- **SPEC-008 — Lint e Testes do Frontend** — `P0` — Frontend / Qualidade
- **SPEC-009 — Segurança de Dependências Rust e Supply Chain** — `P1` — Security / Dependencies
- **SPEC-010 — Hardening do Tauri: CSP e Capabilities** — `P0` — Security / Tauri
- **SPEC-011 — Integridade de Releases e Assinatura de Código** — `P1` — Release / Trust
- **SPEC-012 — Modularização de App.tsx e Orquestração Frontend** — `P0` — Frontend / Arquitetura
- **SPEC-013 — Modelo de Estado do Trainer e Polling** — `P1` — Frontend / Runtime UX
- **SPEC-014 — Design System e Tipografia** — `P1` — Frontend / Visual
- **SPEC-015 — Hierarquia Semântica de Ações** — `P1` — Frontend / UX
- **SPEC-016 — Status Operacional: Processo, Build e Capacidade** — `P1` — Frontend / UX
- **SPEC-017 — Refinamento de Identidade Visual DRG** — `P2` — Frontend / Visual
- **SPEC-018 — Responsividade e Contrato da Janela Tauri** — `P1` — Frontend / Desktop UX
- **SPEC-019 — Estados, Feedback e Tratamento de Erros no Frontend** — `P1` — Frontend / UX
- **SPEC-020 — Preservação de Save e Transações de Mutação Permanente** — `P0` — Safety / Save
- **SPEC-021 — Documentação Arquitetural e Contratos de Manutenção** — `P1` — Documentation
- **SPEC-022 — Matriz de Regressão e Critérios de Release 9+** — `P1` — Governança / Release

## Arquivos auxiliares

- `ROADMAP.md` — ordem recomendada de implementação.
- `TRACEABILITY.md` — rastreia cada ponto da avaliação para uma ou mais SPECs.
- `SOURCE-ASSESSMENT.md` — cópia do markdown usado como origem.
- `DRG-TRAINER-SPECS-COMPLETE.md` — todas as SPECs em um único arquivo.

## Princípios que não devem ser perdidos

- Segurança e isolamento do acesso Win32.
- Validação da build antes de operações dependentes de offsets.
- Backup antes de mutações permanentes.
- UI densa e rápida.
- Paleta carvão/cinza/âmbar/verde.
- Identidade de painel industrial + ferramenta de engenharia + DRG.
- Polimento incremental em vez de redesign.


---

# DRG-TRAINER — Roadmap de Implementação das SPECs

## Objetivo

Levar o projeto do estado avaliado (~8,1/10 geral; ~8,3/10 visual) para uma base de nível 9+ sem perder os pontos fortes atuais: segurança de memória, validação de build, densidade da UI, identidade industrial e simplicidade operacional.

## Fase 1 — Fundamentos bloqueantes (P0)

1. **SPEC-001 — Invariantes de Segurança de Memória**
2. **SPEC-002 — BuildProfile e Compatibilidade Multi-Build**
3. **SPEC-003 — Workflow de Atualização de Builds do DRG**
4. **SPEC-004 — Modularização do Backend Rust**
5. **SPEC-005 — Decomposição do Unreal Runtime**
6. **SPEC-006 — Arquitetura de Testes Offline e Live-Game**
7. **SPEC-007 — Quality Gates no CI/CD**
8. **SPEC-008 — Lint e Testes do Frontend**
9. **SPEC-010 — Hardening do Tauri: CSP e Capabilities**
10. **SPEC-012 — Modularização de App.tsx e Orquestração Frontend**
11. **SPEC-020 — Preservação de Save e Transações de Mutação Permanente**

## Fase 2 — Maturidade e produto (P1)

1. **SPEC-009 — Segurança de Dependências Rust e Supply Chain**
2. **SPEC-011 — Integridade de Releases e Assinatura de Código**
3. **SPEC-013 — Modelo de Estado do Trainer e Polling**
4. **SPEC-014 — Design System e Tipografia**
5. **SPEC-015 — Hierarquia Semântica de Ações**
6. **SPEC-016 — Status Operacional: Processo, Build e Capacidade**
7. **SPEC-018 — Responsividade e Contrato da Janela Tauri**
8. **SPEC-019 — Estados, Feedback e Tratamento de Erros no Frontend**
9. **SPEC-021 — Documentação Arquitetural e Contratos de Manutenção**
10. **SPEC-022 — Matriz de Regressão e Critérios de Release 9+**

## Fase 3 — Refinamento visual (P2)

1. **SPEC-017 — Refinamento de Identidade Visual DRG**

## Sequenciamento prático sugerido

1. SPEC-001 → SPEC-002 → SPEC-003: estabilizar a compatibilidade de memória/build.
2. SPEC-004 → SPEC-005 → SPEC-020: modularizar backend sem perder as garantias atuais.
3. SPEC-006 → SPEC-008 → SPEC-007 → SPEC-009: criar testes offline e transformar CI em gate real.
4. SPEC-010 → SPEC-011: hardening e confiança de distribuição.
5. SPEC-012 → SPEC-013 → SPEC-019: modularizar orchestration e tornar estados previsíveis.
6. SPEC-014 → SPEC-015 → SPEC-016 → SPEC-018 → SPEC-017: polimento visual, sem redesign.
7. SPEC-021 → SPEC-022: consolidar documentação, regressão e critérios de release.

## Regra de execução

Cada SPEC deve ser implementada isoladamente, preferencialmente em branch/PR própria. Uma SPEC só é considerada concluída quando seus critérios de aceite e Definition of Done estiverem satisfeitos.


---

# Matriz de Rastreabilidade — Avaliação → SPEC

| Achado / recomendação da avaliação | SPEC |
|---|---|
| Handles RAII, leitura/escrita exata e isolamento Win32 devem ser preservados | SPEC-001 |
| Hash + offsets + signatures representam um perfil de build | SPEC-002 |
| Sobreviver a dezenas de atualizações do DRG | SPEC-003 |
| `memory.rs` e backend crescendo | SPEC-004 |
| `unreal_runtime.rs` como hotspot | SPEC-005 |
| Testes dependem do jogo aberto/build exata | SPEC-006 |
| CI sem `cargo test`, `clippy` e audit Rust | SPEC-007 / SPEC-009 |
| Frontend sem lint/testes suficientes | SPEC-008 |
| `csp: null` | SPEC-010 |
| Checksums já bons; falta assinatura/provenance | SPEC-011 |
| `App.tsx` concentra polling, estado, ações e hotkeys | SPEC-012 |
| Polling/estado precisam crescer com multi-build | SPEC-013 |
| Tipografia pequena (7–11 px) | SPEC-014 |
| Botões runtime/normal/permanente têm destaque semelhante | SPEC-015 |
| Attached/build verified deve ser mais evidente | SPEC-016 |
| Reforçar identidade DRG sem copiar WeMod | SPEC-017 |
| Breakpoint 700 px abaixo da minWidth Tauri ~760 px | SPEC-018 |
| Refinar estados/interações | SPEC-019 |
| Backup de save é força atual e deve virar contrato | SPEC-020 |
| Documentação já boa, mas faltam contratos arquiteturais duráveis | SPEC-021 |
| Objetivo 9+ precisa de critérios mensuráveis de release | SPEC-022 |


---

# SPEC-001 — Invariantes de Segurança de Memória

**Prioridade:** P0
**Área:** Backend / Rust / Safety
**Status inicial:** Proposed

## Contexto / problema

O projeto já possui boas proteções em torno de handles Win32, leitura/escrita exata e separação entre acesso read-only e write. Essas garantias precisam virar contrato arquitetural explícito para não regredirem durante a expansão do trainer.

## Proposta

Formalizar uma camada única de acesso a processo e memória, tratada como boundary de segurança do backend. Toda chamada Win32 relacionada a processo, leitura, escrita, alocação ou execução remota deve permanecer encapsulada nessa camada. Nenhum módulo de domínio poderá chamar APIs Win32 diretamente.

Invariantes obrigatórios: handles RAII com `Drop`; abertura read-only por padrão; escrita somente por API explícita; validação de bytes lidos/escritos; checagem de ponteiros/endereço antes de dereference; erros tipados; nenhuma escrita silenciosa; operações nativas condicionadas a build validada.

## Escopo

- Consolidar wrapper de processo/memória.
- Proibir Win32 direto fora do módulo de infraestrutura.
- Padronizar erros de leitura/escrita.
- Criar testes unitários para validação de parâmetros e erros.

## Arquivos / áreas prováveis

- `src-tauri/src/memory/process.rs`
- `src-tauri/src/memory.rs`
- `src-tauri/src/error.rs (novo, se necessário)`

## Critérios de aceite

- [ ] Nenhuma API Win32 de memória/processo é chamada fora da camada autorizada.
- [ ] Todo handle é fechado por RAII.
- [ ] Toda escrita valida quantidade exata de bytes.
- [ ] Falha de escrita nunca é reportada como sucesso.
- [ ] `cargo test` cobre contratos que não dependem do jogo.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Refatoração pode alterar assinaturas internas.
- Excesso de abstração pode dificultar debugging; manter wrappers finos.

## Dependências

- Nenhuma dependência obrigatória.

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-002 — BuildProfile e Compatibilidade Multi-Build

**Prioridade:** P0
**Área:** Arquitetura / Compatibilidade
**Status inicial:** Proposed

## Contexto / problema

Hash do executável, offsets, endereços, assinaturas e funções nativas representam hoje, na prática, um único perfil de build. A continuidade do projeto depende de desacoplar esses dados da infraestrutura genérica.

## Proposta

Introduzir `BuildProfile` como entidade central de compatibilidade. Cada versão suportada do DRG terá um perfil imutável contendo SHA-256 esperado, offsets, signatures e native functions. O runtime resolve o perfil pela hash do executável antes de expor recursos dependentes de memória.

Estrutura proposta:

```text
src-tauri/src/build_profiles/
├── mod.rs
├── registry.rs
├── types.rs
└── profiles/
    ├── build_<id>.rs
    └── ...
```

Interface conceitual:

```rust
struct BuildProfile {
    id: &'static str,
    executable_sha256: &'static str,
    offsets: Offsets,
    signatures: Signatures,
    native_functions: NativeFunctions,
}
```

O registry retorna `Supported(profile)` ou `Unsupported { sha256 }`.

## Escopo

- Mover constantes específicas de build para perfis.
- Criar registry por hash.
- Expor identificação da build ao frontend.
- Permitir mais de um perfil simultaneamente.
- Bloquear operações incompatíveis quando nenhum perfil corresponder.

## Arquivos / áreas prováveis

- `src-tauri/src/build_profiles/* (novo)`
- `src-tauri/src/memory.rs`
- `src-tauri/src/unreal_runtime.rs`
- `src-tauri/src/lib.rs`

## Critérios de aceite

- [ ] Adicionar uma segunda build não exige editar lógica genérica de leitura/escrita.
- [ ] Build desconhecida é detectada e bloqueada antes de operações dependentes de offsets.
- [ ] Frontend recebe `profile_id`, hash e estado de suporte.
- [ ] Perfis possuem testes de consistência.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Duplicação entre perfis; criar tipos compartilhados.
- Offsets incompletos por build; permitir capability flags por perfil.

## Dependências

- SPEC-001

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-003 — Workflow de Atualização de Builds do DRG

**Prioridade:** P0
**Área:** Manutenção / Compatibilidade
**Status inicial:** Proposed

## Contexto / problema

Mesmo com `BuildProfile`, o projeto precisa de um processo repetível para absorver atualizações do jogo sem transformar cada patch em investigação ad hoc.

## Proposta

Criar procedimento documentado de onboarding de nova build: identificar hash, duplicar perfil anterior, marcar capacidades como não verificadas, atualizar offsets/signatures, executar validações offline, rodar live tests, registrar evidências e só então promover o perfil a `supported`.

Estados sugeridos por perfil: `draft`, `verified`, `supported`, `deprecated`. O binário público deve incluir apenas perfis `supported`, salvo build de desenvolvimento.

## Escopo

- Checklist de atualização de build.
- Manifesto de capacidades verificadas.
- Registro de hash e data.
- Live-test gate antes de release.
- Template de PR para nova build.

## Arquivos / áreas prováveis

- `docs/build-support.md (novo)`
- `docs/templates/build-profile-checklist.md (novo)`
- `src-tauri/src/build_profiles/*`
- `.github/PULL_REQUEST_TEMPLATE.md`

## Critérios de aceite

- [ ] Há procedimento reproduzível para suportar nova build.
- [ ] Toda nova build tem evidência de validação.
- [ ] Capacidade não verificada permanece desabilitada.
- [ ] Release notes indicam builds suportadas.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Processo manual pode ser ignorado; CI deve validar metadados mínimos.

## Dependências

- SPEC-002
- SPEC-006
- SPEC-007

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-004 — Modularização do Backend Rust

**Prioridade:** P0
**Área:** Backend / Arquitetura
**Status inicial:** Proposed

## Contexto / problema

`memory.rs` e módulos relacionados já acumulam responsabilidades. O crescimento de novas funcionalidades pode aumentar acoplamento e custo cognitivo.

## Proposta

Reorganizar o backend em camadas com responsabilidades explícitas, sem introduzir arquitetura cerimonial.

Proposta:

```text
src-tauri/src/
├── app/
│   ├── commands.rs
│   └── state.rs
├── domain/
│   ├── inventory/
│   ├── progression/
│   ├── weapons/
│   └── player/
├── infrastructure/
│   ├── process/
│   ├── memory/
│   ├── save/
│   └── unreal/
├── build_profiles/
└── shared/
```

Commands Tauri apenas validam input, chamam serviços e transformam erros. Regras de domínio não conhecem Tauri nem Win32.

## Escopo

- Separar comandos Tauri de domínio.
- Mover acesso Win32 para infrastructure.
- Manter módulos de domínio pequenos e coesos.
- Evitar dependências circulares.

## Arquivos / áreas prováveis

- `src-tauri/src/lib.rs`
- `src-tauri/src/memory.rs`
- `src-tauri/src/progression.rs`
- `src-tauri/src/resources.rs`
- `src-tauri/src/weapons.rs`
- `src-tauri/src/save_reader.rs`

## Critérios de aceite

- [ ] Nenhum command contém regra de domínio complexa.
- [ ] Domínio pode ser testado sem Tauri.
- [ ] Infraestrutura depende de tipos compartilhados, não o contrário.
- [ ] Módulos principais possuem responsabilidade única claramente documentada.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Big-bang refactor; executar por slices e manter compatibilidade de APIs Tauri.

## Dependências

- SPEC-001
- SPEC-002

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-005 — Decomposição do Unreal Runtime

**Prioridade:** P0
**Área:** Backend / Unreal
**Status inicial:** Proposed

## Contexto / problema

`unreal_runtime.rs` concentra lógica suficiente para se tornar um hotspot de manutenção, principalmente quando novas features e builds forem adicionadas.

## Proposta

Dividir o runtime Unreal por responsabilidade: resolução de objetos, traversal, validação, native calls, caches e operações específicas do jogo.

```text
infrastructure/unreal/
├── mod.rs
├── resolver.rs
├── object.rs
├── validation.rs
├── native_call.rs
├── cache.rs
└── types.rs
```

Código específico de DRG deve ficar nos módulos de domínio ou adaptadores DRG, não no resolver genérico.

## Escopo

- Extrair resolução de ponteiros.
- Extrair validação de objetos.
- Isolar native calls.
- Definir cache com invalidation explícita.
- Reduzir dependência de constantes globais.

## Arquivos / áreas prováveis

- `src-tauri/src/unreal_runtime.rs`
- `src-tauri/src/infrastructure/unreal/* (novo)`

## Critérios de aceite

- [ ] Nenhum arquivo do runtime concentra múltiplos eixos de responsabilidade.
- [ ] Caches são invalidados em detach/restart/build change.
- [ ] Native calls exigem build/profile verificado.
- [ ] Testes unitários cobrem resolvers puros quando possível.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Alterar timing do runtime; preservar comportamento e medir regressões.

## Dependências

- SPEC-002
- SPEC-004

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-006 — Arquitetura de Testes Offline e Live-Game

**Prioridade:** P0
**Área:** Qualidade / Testes
**Status inicial:** Proposed

## Contexto / problema

Os testes atuais exercitam funcionalidades reais, mas muitos dependem do DRG aberto e da build correta, o que impede uma regression suite confiável no CI.

## Proposta

Criar três níveis de teste:

1. `unit`: parsers, validações, catálogos, cálculo de endereços, state machines e regras de domínio; sempre offline.
2. `integration-offline`: módulos integrados com fake/mocked process memory.
3. `live-game`: acessa processo real; marcado `#[ignore]` ou feature `live-tests`.

Introduzir uma trait de memória (`MemoryReader`, `MemoryWriter` ou equivalente) para permitir fake backend em testes sem contaminar produção com mocks.

## Escopo

- Classificar testes existentes.
- Criar fake memory backend.
- Mover live tests para módulo específico.
- Adicionar fixtures quando aplicável.
- Documentar execução local.

## Arquivos / áreas prováveis

- `src-tauri/src/memory/tests.rs`
- `src-tauri/tests/* (novo)`
- `src-tauri/src/testing/* (opcional)`

## Critérios de aceite

- [ ] `cargo test` roda em máquina sem DRG instalado.
- [ ] Live tests não rodam por padrão.
- [ ] Regras de domínio críticas têm testes determinísticos.
- [ ] Falhas de perfil/build possuem testes offline.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Mocks irreais; priorizar fakes simples baseados em buffers e fixtures reais anonimizadas.

## Dependências

- SPEC-001
- SPEC-004

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-007 — Quality Gates no CI/CD

**Prioridade:** P0
**Área:** CI/CD
**Status inicial:** Proposed

## Contexto / problema

O CI já executa build, audit npm, fmt e cargo check, mas ainda não fecha o ciclo de qualidade com testes, clippy e auditoria Rust.

## Proposta

Transformar o workflow em gates independentes e obrigatórios: frontend, rust-format, rust-lint, rust-test, dependency-audit, codeql e build.

Comandos mínimos:

```text
npm ci
npm run build
npm run lint
npm test -- --run
npm audit --audit-level=high
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo audit
```

Manter Actions pinadas por commit SHA.

## Escopo

- Adicionar jobs separados.
- Cache seguro para Cargo/npm.
- Falhar PR em warning de Clippy.
- Publicar artefatos somente após gates.
- Manter CodeQL e Dependabot.

## Arquivos / áreas prováveis

- `.github/workflows/build.yml`
- `.github/workflows/security.yml (opcional)`
- `package.json`

## Critérios de aceite

- [ ] PR não pode passar com teste/lint falhando.
- [ ] `cargo test` executa sem jogo.
- [ ] Clippy warnings são erros.
- [ ] Dependências vulneráveis em nível definido bloqueiam pipeline.
- [ ] Release depende dos mesmos gates.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- `cargo audit` pode gerar ruído; documentar política de exceções temporárias.

## Dependências

- SPEC-006
- SPEC-008

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-008 — Lint e Testes do Frontend

**Prioridade:** P0
**Área:** Frontend / Qualidade
**Status inicial:** Proposed

## Contexto / problema

O frontend possui build funcional, mas não há uma camada equivalente de lint e testes para proteger comportamento e refatorações.

## Proposta

Adicionar ESLint com regras TypeScript/React, testes com Vitest e Testing Library e cobertura focada em comportamento. Não perseguir cobertura percentual artificial.

Prioridades de teste: renderização por estado de conexão, habilitação/desabilitação de ações, confirmação de mutações permanentes, hotkeys, tratamento de erro e mudança de tab.

## Escopo

- Configurar ESLint.
- Configurar Vitest/Testing Library.
- Criar helpers de mock para comandos Tauri.
- Adicionar scripts npm.
- Integrar ao CI.

## Arquivos / áreas prováveis

- `package.json`
- `eslint.config.*`
- `vitest.config.*`
- `src/**/*.test.tsx`

## Critérios de aceite

- [ ] `npm run lint` e `npm test -- --run` funcionam localmente e no CI.
- [ ] Ações permanentes têm testes de confirmação.
- [ ] Estados attached/unsupported/detached têm testes.
- [ ] Mocks Tauri ficam centralizados.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Testar implementação em vez de comportamento; preferir queries por papel/texto/estado.

## Dependências

- SPEC-012
- SPEC-016

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-009 — Segurança de Dependências Rust e Supply Chain

**Prioridade:** P1
**Área:** Security / Dependencies
**Status inicial:** Proposed

## Contexto / problema

O projeto já usa Dependabot, CodeQL, `npm audit` e Actions pinadas por SHA, mas a cadeia Rust pode receber controles equivalentes.

## Proposta

Adicionar `cargo audit` ou solução equivalente, política explícita para advisories, lockfile obrigatório em CI e revisão periódica de dependências. Preservar pinning de GitHub Actions por commit e limitar permissões de workflows ao mínimo.

## Escopo

- Auditoria Rust.
- Permissions mínimas no GitHub Actions.
- Política de exceção de advisory.
- Revisão de dependências não usadas.

## Arquivos / áreas prováveis

- `.github/workflows/*`
- `src-tauri/Cargo.lock`
- `SECURITY.md`
- `deny.toml (opcional)`

## Critérios de aceite

- [ ] CI detecta advisories Rust.
- [ ] Workflows possuem `permissions` explícitas.
- [ ] Exceções têm justificativa e prazo.
- [ ] Lockfiles são usados em builds/release.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Advisories transitórios sem patch; permitir exceção documentada.

## Dependências

- SPEC-007

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-010 — Hardening do Tauri: CSP e Capabilities

**Prioridade:** P0
**Área:** Security / Tauri
**Status inicial:** Proposed

## Contexto / problema

A capability `core:default` é restritiva e positiva, mas `csp: null` reduz defesa em profundidade para uma WebView cujo backend possui acesso sensível ao processo do jogo.

## Proposta

Definir CSP mínima compatível com Vite/Tauri, bloqueando conteúdo remoto por padrão. Revisar capabilities comando a comando e garantir que filesystem, shell, opener ou HTTP não sejam liberados sem necessidade.

Exemplo conceitual de política: `default-src 'self'; img-src 'self' asset: data:; style-src 'self' 'unsafe-inline'; script-src 'self'` — a política final deve ser validada contra o build real e não copiada cegamente.

## Escopo

- Ativar CSP.
- Inventariar capabilities.
- Remover permissões não usadas.
- Documentar rationale de cada permissão.

## Arquivos / áreas prováveis

- `src-tauri/tauri.conf.json`
- `src-tauri/capabilities/default.json`
- `SECURITY.md`

## Critérios de aceite

- [ ] Aplicativo inicia e funciona com CSP ativa.
- [ ] Nenhuma permissão Tauri não utilizada permanece habilitada.
- [ ] Conteúdo remoto não é executável por padrão.
- [ ] Mudança de capability exige revisão explícita.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- CSP quebrar assets/fontes; ajustar somente o necessário.

## Dependências

- Nenhuma dependência obrigatória.

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-011 — Integridade de Releases e Assinatura de Código

**Prioridade:** P1
**Área:** Release / Trust
**Status inicial:** Proposed

## Contexto / problema

A release já publica executável e `SHA256SUMS.txt`, o que é bom. Para distribuição mais ampla falta provenance automatizada e, futuramente, assinatura de código Windows.

## Proposta

Automatizar geração de checksums no pipeline, anexar manifest de build e documentar verificação. Preparar o pipeline para assinatura Authenticode quando houver certificado apropriado, mantendo secrets somente no ambiente de release.

Opcionalmente adicionar provenance/SBOM (CycloneDX ou SPDX) quando o projeto atingir distribuição maior.

## Escopo

- Checksum automatizado.
- Manifesto com commit/tag/build profile.
- Preparar stage de code signing.
- Documentar verificação.

## Arquivos / áreas prováveis

- `.github/workflows/release.yml`
- `docs/release.md`
- `CHANGELOG.md`

## Critérios de aceite

- [ ] Toda release possui checksum gerado pelo CI.
- [ ] Artefato registra commit e perfis suportados.
- [ ] Nenhuma chave/certificado fica no repositório.
- [ ] Assinatura pode ser ligada sem redesenhar pipeline.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Certificado tem custo e operação própria; tratar como fase posterior.

## Dependências

- SPEC-003
- SPEC-007
- SPEC-009

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-012 — Modularização de App.tsx e Orquestração Frontend

**Prioridade:** P0
**Área:** Frontend / Arquitetura
**Status inicial:** Proposed

## Contexto / problema

`App.tsx` concentra polling, estado, ações, hotkeys e coordenação. As seções visuais já estão separadas, então o próximo gargalo é a orchestration.

## Proposta

Reduzir `App.tsx` ao papel de composition root. Extrair hooks e serviços orientados a responsabilidade:

```text
src/
├── hooks/
│   ├── useTrainerConnection.ts
│   ├── useTrainerPolling.ts
│   ├── useInventoryActions.ts
│   ├── useProgressionActions.ts
│   ├── useWeaponRuntime.ts
│   └── useTrainerHotkeys.ts
├── services/
│   └── trainer-api.ts
├── state/
│   └── trainer-state.ts
└── components/
```

Chamadas `invoke` devem ser centralizadas em `trainer-api.ts`.

## Escopo

- Extrair polling.
- Extrair hotkeys.
- Centralizar comandos Tauri.
- Separar estado de domínio de estado visual.
- Manter componentes de seção majoritariamente presentacionais.

## Arquivos / áreas prováveis

- `src/App.tsx`
- `src/hooks/* (novo)`
- `src/services/trainer-api.ts (novo)`
- `src/components/*`

## Critérios de aceite

- [ ] `App.tsx` vira composition root enxuto.
- [ ] Nenhum componente chama `invoke` diretamente, salvo camada designada.
- [ ] Polling tem lifecycle e cancelamento previsíveis.
- [ ] Hooks possuem testes independentes.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Criar hooks excessivamente genéricos; manter cada hook ligado a um caso de uso concreto.

## Dependências

- SPEC-008

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-013 — Modelo de Estado do Trainer e Polling

**Prioridade:** P1
**Área:** Frontend / Runtime UX
**Status inicial:** Proposed

## Contexto / problema

Polling e estado de conexão são centrais para a experiência, mas crescerão em complexidade quando houver múltiplas builds, erros de runtime e capacidades condicionais.

## Proposta

Representar explicitamente estados do trainer em vez de booleanos dispersos.

```ts
type TrainerStatus =
  | { kind: "detached" }
  | { kind: "attaching" }
  | { kind: "unsupported"; sha256: string }
  | { kind: "attached"; profileId: string; verified: true }
  | { kind: "error"; code: string; recoverable: boolean };
```

O polling deve respeitar foco/lifecycle, impedir chamadas concorrentes e usar backoff leve em erro.

## Escopo

- State machine de conexão.
- Polling sem overlap.
- Cancelamento em unmount.
- Capabilities por build.
- Derivar UI a partir do estado.

## Arquivos / áreas prováveis

- `src/hooks/useTrainerConnection.ts`
- `src/hooks/useTrainerPolling.ts`
- `src/state/trainer-state.ts`

## Critérios de aceite

- [ ] Não existem dois polls concorrentes para o mesmo recurso.
- [ ] Unsupported build é um estado de primeira classe.
- [ ] Ações derivam `disabled` do estado central.
- [ ] Erro recuperável possui retry previsível.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- State machine excessiva; manter apenas estados observáveis reais.

## Dependências

- SPEC-002
- SPEC-012

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-014 — Design System e Tipografia

**Prioridade:** P1
**Área:** Frontend / Visual
**Status inicial:** Proposed

## Contexto / problema

A interface é consistente e tem boa paleta, mas existem textos de 7–11 px que podem perder legibilidade em 1080p com scaling do Windows.

## Proposta

Preservar a estética densa, mas formalizar tokens de tipografia, spacing, radius e cores. Subir discretamente os tamanhos mínimos.

Meta sugerida:

```text
metadata secundário   9px
metadata normal      10px
option name          12px
section title        11px
button/hotkey        10px
valor principal      12–13px
```

Criar tokens CSS e evitar valores mágicos repetidos.

## Escopo

- Tokens de font-size/line-height.
- Escala de spacing.
- Estados disabled/hover/focus consistentes.
- Contraste verificável.
- Manter densidade.

## Arquivos / áreas prováveis

- `src/styles.css`
- `src/components/ui/*`

## Critérios de aceite

- [ ] Nenhum texto funcional crítico abaixo do mínimo definido.
- [ ] UI mantém mesma densidade aproximada.
- [ ] Focus ring é visível.
- [ ] Contraste de texto e controles é adequado.
- [ ] Valores de tipografia principais vêm de tokens.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Aumento de fonte causar overflow; ajustar grid e min-width junto com SPEC-018.

## Dependências

- Nenhuma dependência obrigatória.

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-015 — Hierarquia Semântica de Ações

**Prioridade:** P1
**Área:** Frontend / UX
**Status inicial:** Proposed

## Contexto / problema

Ações runtime, edição simples e mutações permanentes usam destaque visual semelhante, reduzindo a percepção de impacto.

## Proposta

Introduzir três classes semânticas de ação: `runtime`, `standard` e `persistent`. O componente de ação recebe uma semântica explícita e aplica label, ícone, confirmação e tratamento visual coerente.

```text
Runtime     Infinite Magazine     [toggle]
Standard    Edit Credits          [Edit]
Persistent  Unlock All Weapons    [PERMANENT] [Unlock all]
```

Vermelho deve ficar reservado para erro/destrutivo; mutação persistente pode continuar em âmbar reforçado.

## Escopo

- Criar semântica de ação.
- Badge `PERMANENT`.
- Confirmação para mutações persistentes.
- Ícone de save/estado quando aplicável.
- Padronizar copy.

## Arquivos / áreas prováveis

- `src/components/*`
- `src/styles.css`
- `src/types/actions.ts (novo)`

## Critérios de aceite

- [ ] Toda ação possui semântica declarada.
- [ ] Mutações persistentes são distinguíveis antes do clique.
- [ ] Ações de runtime não parecem destrutivas.
- [ ] Confirmações são testadas.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Excesso de badges; mostrar apenas quando a semântica adiciona informação.

## Dependências

- SPEC-008
- SPEC-014

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-016 — Status Operacional: Processo, Build e Capacidade

**Prioridade:** P1
**Área:** Frontend / UX
**Status inicial:** Proposed

## Contexto / problema

PID, attached e verificação de build existem, mas podem ser lidos de forma mais imediata. Em um trainer, saber se é seguro operar é informação prioritária.

## Proposta

Criar um status bar/header compacto com estado operacional explícito:

```text
● GAME ATTACHED    ✓ BUILD VERIFIED
FSD-Win64-Shipping.exe · PID 26248 · profile <id>
```

Para build desconhecida:

```text
○ GAME ATTACHED    ! BUILD UNSUPPORTED
Memory-dependent actions disabled
```

Tooltips ficam para detalhes, não para informação essencial.

## Escopo

- Estado attached/detached.
- Build verified/unsupported.
- PID e profile ID secundários.
- Estados de erro.
- Desabilitação coerente das ações.

## Arquivos / áreas prováveis

- `src/App.tsx ou header extraído`
- `src/components/trainer-status.tsx (novo)`
- `src/styles.css`

## Critérios de aceite

- [ ] Usuário entende em menos de um olhar se pode operar.
- [ ] Unsupported build não depende apenas de tooltip.
- [ ] Status é acessível por texto, não apenas cor/ícone.
- [ ] Todas as ações incompatíveis ficam disabled.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Header ocupar espaço; manter altura compacta.

## Dependências

- SPEC-002
- SPEC-013
- SPEC-014

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-017 — Refinamento de Identidade Visual DRG

**Prioridade:** P2
**Área:** Frontend / Visual
**Status inicial:** Proposed

## Contexto / problema

A UI já combina carvão, cinza quente, âmbar e verde e tem identidade coerente, mas ainda pode parecer mais produto dedicado ao DRG sem copiar WeMod/Aurora.

## Proposta

Preservar a linguagem de painel industrial + ferramenta de engenharia + DRG. Adicionar identidade apenas em elementos de baixo ruído: header, pattern industrial sutil, separadores, microtexturas vetoriais, ícones geométricos inspirados nas classes/equipamentos e nomenclatura temática.

Evitar artwork grande, neon, glassmorphism e excesso de decoração.

## Escopo

- Header mais proprietário.
- Background/pattern discreto.
- Ícones consistentes.
- Microinterações industriais discretas.
- Sem uso de assets protegidos sem licença adequada.

## Arquivos / áreas prováveis

- `src/styles.css`
- `src/assets/*`
- `src/components/header/*`

## Critérios de aceite

- [ ] Interface continua rápida e densa.
- [ ] Identidade DRG aumenta sem comprometer legibilidade.
- [ ] Nenhuma decoração compete com controles.
- [ ] Assets têm origem/licença documentada.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Overdesign; revisão visual deve comparar antes/depois em 940x660.

## Dependências

- SPEC-014
- SPEC-016

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-018 — Responsividade e Contrato da Janela Tauri

**Prioridade:** P1
**Área:** Frontend / Desktop UX
**Status inicial:** Proposed

## Contexto / problema

A janela mínima é aproximadamente 760 px, enquanto o breakpoint de uma coluna está em 700 px, portanto o comportamento responsivo pode nunca ser atingido no uso normal.

## Proposta

Alinhar breakpoints ao contrato real da janela. Testar explicitamente 760x590, 820x620, 940x660 e resoluções maiores. Em largura compacta, reduzir spacing e migrar grids selecionados para uma coluna antes de comprometer legibilidade.

Breakpoint inicial sugerido: `@media (max-width: 820px)`, sujeito a teste visual.

## Escopo

- Revisar minWidth/minHeight.
- Ajustar breakpoint.
- Prevenir overflow horizontal.
- Testar scaling 100%, 125% e 150%.
- Manter navegação utilizável.

## Arquivos / áreas prováveis

- `src-tauri/tauri.conf.json`
- `src/styles.css`
- `src/components/*`

## Critérios de aceite

- [ ] Sem overflow horizontal na largura mínima suportada.
- [ ] Textos não colidem em 125% scaling.
- [ ] A UI reorganiza antes de ficar ilegível.
- [ ] Screenshots de regressão cobrem tamanhos-alvo.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- WebView scaling varia por sistema; validar em Windows real.

## Dependências

- SPEC-014

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-019 — Estados, Feedback e Tratamento de Erros no Frontend

**Prioridade:** P1
**Área:** Frontend / UX
**Status inicial:** Proposed

## Contexto / problema

O refinamento visual depende também de estados e interações: loading, sucesso, falha, operação bloqueada e confirmação precisam de linguagem consistente.

## Proposta

Padronizar estados assíncronos por ação: `idle`, `pending`, `success`, `error`, `blocked`. Botões devem impedir duplo clique enquanto pending. Toasts devem informar resultado sem substituir estados persistentes importantes. Erros do backend devem ter códigos estáveis mapeados para mensagens claras.

Para operações persistentes: confirmar → executar → verificar → informar backup/resultados.

## Escopo

- Loading state por ação.
- Debounce/double-submit prevention.
- Error mapping.
- Toast system.
- Feedback de backup/save mutation.

## Arquivos / áreas prováveis

- `src/components/ui/*`
- `src/services/trainer-api.ts`
- `src/state/*`
- `src-tauri/src/error.rs`

## Critérios de aceite

- [ ] Nenhuma ação assíncrona pode disparar duplicada inadvertidamente.
- [ ] Erros conhecidos têm mensagem específica.
- [ ] Blocked state explica motivo.
- [ ] Mutações persistentes informam resultado e backup.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Toasts em excesso; reservar para eventos relevantes.

## Dependências

- SPEC-012
- SPEC-013
- SPEC-015

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-020 — Preservação de Save e Transações de Mutação Permanente

**Prioridade:** P0
**Área:** Safety / Save
**Status inicial:** Proposed

## Contexto / problema

Backup antes de alterações permanentes é um ponto forte atual e deve ser formalizado para que novas features não contornem essa proteção.

## Proposta

Criar serviço único para mutações persistentes. Toda alteração de save segue pipeline: localizar save → validar → criar backup versionado → executar mutação → reler/verificar → reportar resultado. Em falha após backup, preservar artefatos e fornecer caminho de restauração.

Nenhum comando de domínio pode escrever save diretamente fora desse serviço.

## Escopo

- Centralizar backup.
- Nome/versionamento de backup.
- Verificação pós-escrita.
- Erro transacional.
- Documentar restauração.

## Arquivos / áreas prováveis

- `src-tauri/src/save_reader.rs`
- `src-tauri/src/infrastructure/save/* (novo)`
- `src-tauri/src/domain/*`

## Critérios de aceite

- [ ] Toda mutação persistente cria backup antes de escrever.
- [ ] Backup não é sobrescrito silenciosamente.
- [ ] Falha pós-escrita é detectada.
- [ ] Há instrução de restauração testada.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Espaço em disco; política de retenção simples e documentada.

## Dependências

- SPEC-004

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-021 — Documentação Arquitetural e Contratos de Manutenção

**Prioridade:** P1
**Área:** Documentation
**Status inicial:** Proposed

## Contexto / problema

O repositório já possui README, changelog e security policy, mas a maturidade futura depende de registrar decisões como BuildProfile, boundary Win32, testes live-game e política de release.

## Proposta

Criar documentação curta e operacional, não documentação duplicada do código. Adotar ADRs somente para decisões arquiteturais duráveis e controversas.

```text
docs/
├── architecture.md
├── build-support.md
├── testing.md
├── release.md
├── frontend-design.md
└── adr/
```

Cada ADR registra contexto, decisão, alternativas e consequências.

## Escopo

- Mapa de arquitetura.
- Guia de builds.
- Guia de testes.
- Contrato visual.
- ADRs das decisões centrais.

## Arquivos / áreas prováveis

- `docs/*`
- `README.md`
- `CONTRIBUTING.md (novo, se necessário)`

## Critérios de aceite

- [ ] Novo contribuidor entende onde colocar uma feature.
- [ ] Há instrução clara para atualizar build.
- [ ] Há distinção entre unit/integration/live tests.
- [ ] Decisões críticas têm ADR.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Docs desatualizadas; ligar mudanças de arquitetura a checklist de PR.

## Dependências

- SPEC-002
- SPEC-004
- SPEC-006
- SPEC-010

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.


---

# SPEC-022 — Matriz de Regressão e Critérios de Release 9+

**Prioridade:** P1
**Área:** Governança / Release
**Status inicial:** Proposed

## Contexto / problema

A avaliação atual é aproximadamente 8,1/10 geral e 8,3/10 visual. Para transformar a melhoria em objetivo verificável é necessário definir gates de saída, não apenas percepção.

## Proposta

Criar uma matriz de regressão mínima para cada release: builds suportadas, attach/detach, inventory, progression, weapons, runtime toggles, save mutation, backup/restore, unsupported build, hotkeys, responsividade e instalação/execução.

Definir Definition of Done para o marco 1.0: CI verde, nenhum warning Clippy, testes offline independentes do jogo, live test checklist aprovado, CSP ativa, build profile modular, frontend modularizado e release com checksums.

## Escopo

- Checklist de regressão.
- DoD de release.
- Scorecard de qualidade.
- Critérios bloqueantes vs recomendados.

## Arquivos / áreas prováveis

- `docs/release-checklist.md (novo)`
- `docs/quality-scorecard.md (novo)`
- `.github/ISSUE_TEMPLATE/*`

## Critérios de aceite

- [ ] Toda release possui checklist preenchido.
- [ ] Critérios bloqueantes são objetivos.
- [ ] Falha em build-profile verification bloqueia release.
- [ ] Qualidade visual é validada nos tamanhos-alvo.

## Testes e validação

- [ ] Cobrir o comportamento novo com testes automatizados quando aplicável.
- [ ] Confirmar ausência de regressão nas features existentes.
- [ ] Validar o cenário de erro, não apenas o caminho feliz.
- [ ] Executar os quality gates definidos pela SPEC-007 quando ela estiver implementada.

## Riscos / cuidados

- Checklist virar burocracia; manter apenas gates com valor real.

## Dependências

- SPEC-003
- SPEC-007
- SPEC-018
- SPEC-021

## Fora de escopo

- Adicionar novos cheats apenas para justificar a refatoração.
- Redesign completo que descaracterize a interface atual.
- Mudanças de comportamento não relacionadas a esta SPEC.

## Definition of Done

- [ ] Implementação concluída.
- [ ] Testes relevantes passando.
- [ ] Documentação afetada atualizada.
- [ ] Sem regressão conhecida.
- [ ] Critérios de aceite acima atendidos.
