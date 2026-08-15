# Arquitetura

Mapa curto do projeto: onde as coisas ficam, por que ficam ali, e quais regras
não podem ser quebradas sem discussão explícita (SPEC-021).

## Visão geral

```text
src/                 frontend React (Vite + Tailwind + Radix)
src-tauri/src/       backend Rust (Tauri v2 + Win32)
docs/                este pacote de documentação
```

O frontend nunca fala com o Win32. O backend nunca decide layout. A única
fronteira entre os dois são os comandos Tauri, descritos em
`src-tauri/src/domain/dto.rs` e espelhados em `src/types/trainer.ts`.

## Backend: camadas

```text
app             comandos Tauri e atalhos globais
domain          regras do DRG (inventário, progressão, jogador, armas)
infrastructure  Win32, runtime Unreal e transações de save
build_profiles  dados específicos de cada build do jogo
shared          erros tipados e limites de segurança
```

Dependências apontam sempre para dentro:

- `app` → `domain` → `infrastructure` → `build_profiles` / `shared`
- `domain` não conhece Tauri
- `infrastructure` não conhece `domain`
- nenhuma seta aponta para trás

### Invariantes que não mudam

| Invariante | Onde vive | SPEC |
| --- | --- | --- |
| Win32 de processo/memória só em `infrastructure::process` | `infrastructure/process/memory.rs` | SPEC-001 |
| Handles fechados por RAII | `infrastructure/process/handle.rs` | SPEC-001 |
| Leitura ou escrita parcial é erro | `infrastructure/process/access.rs` | SPEC-001 |
| Endereço validado antes do dereference | `access::validate_range` | SPEC-001 |
| Rotina nativa só executa com assinatura conferida | `infrastructure/unreal/native_call.rs` | SPEC-001 |
| Offsets e hashes só em `build_profiles` | `build_profiles/profiles/*.rs` | SPEC-002 |
| Build não suportada bloqueia antes de qualquer leitura | `domain/session.rs` | SPEC-002 |
| Mutação permanente exige backup antes | `infrastructure/save/mod.rs` | SPEC-020 |

### Onde colocar uma feature nova

| O que você quer fazer | Onde | Observação |
| --- | --- | --- |
| Suportar uma build nova do jogo | `build_profiles/profiles/` | Siga [build-support.md](build-support.md) |
| Ler um campo novo do save | `domain/save_game.rs` + offset no perfil | Valide a faixa plausível |
| Adicionar uma ação permanente | `domain/progression/` ou `domain/player/` | Abra uma `SaveTransaction` |
| Adicionar um toggle de runtime | `domain/weapons/` | Publique status em um store |
| Nova travessia genérica do Unreal | `infrastructure/unreal/` | Sem nomes de classes do DRG |
| Novo comando Tauri | `app/commands.rs` | Só valida entrada e delega |

Se a resposta parecer "nos dois lugares", provavelmente falta um tipo em
`shared` ou um campo no `BuildProfile`.

## Modelo de erro

Todo erro que cruza a fronteira Tauri carrega `{ code, message, recoverable }`.

- `code` é o contrato estável (`shared::error::ErrorCode`). Mudar um valor
  existente quebra o frontend.
- `message` é o detalhe técnico, útil para diagnóstico.
- `recoverable` diz se repetir mais tarde pode funcionar sem intervenção.

O frontend traduz o código em `src/lib/errors.ts`; a mensagem do backend fica
como detalhe secundário. Isso mantém a UI em inglês sem apagar o diagnóstico.

## Frontend: camadas

```text
App.tsx              composition root — só liga hooks a seções
hooks/               orquestração (conexão, polling, ações, hotkeys)
services/            única fronteira `invoke`
state/               estado observável derivado do status do backend
components/trainer/  seções e primitivas do trainer
components/ui/       primitivas shadcn/Radix
lib/                 funções puras (formatação, erros)
types/               contratos com o backend e semântica de ação
```

Regras:

- nenhum componente chama `invoke`; só `services/trainer-api.ts`;
- nenhuma tela calcula `disabled` sozinha; usa `blockedReason` de
  `state/trainer-state.ts`;
- polling só via `useTrainerPolling`, que não permite ciclos concorrentes.

## Fluxo de uma mutação permanente

```text
UI → hook de ação → trainer-api → comando Tauri → domínio
                                                    │
                              TrainerSession::writable()   (build verificada?)
                              profile.require(capability)  (verificado nesta build?)
                              runtime.world_context()      (Space Rig carregada?)
                              save_game.read(...)          (estado antes)
                              runtime.verify_native(...)   (assinaturas conferem?)
                              SaveTransaction::begin(...)  ← BACKUP AQUI
                              runtime.call_verified(...)   (mutação)
                              save_game.read(...)          (verificação)
                              persist_to_disk(...)         (SaveToDisk)
```

Qualquer falha depois do backup passa por `transaction.guard`/`verify`, então a
mensagem sempre diz onde estão os artefatos de restauração.

## Decisões arquiteturais

Decisões duráveis e controversas têm ADR em [`adr/`](adr/). Mudanças de rotina
não precisam de ADR — o histórico do Git basta.
