# Architecture Decision Records

ADRs registram decisões **duráveis e controversas** — aquelas cujo "por quê" se
perde no histórico do Git e cuja reversão custa caro.

O que **não** vira ADR: escolha de biblioteca trivial, refatoração de rotina,
ajuste de layout. Para isso o commit basta.

## Formato

Cada ADR tem contexto, decisão, alternativas consideradas e consequências —
inclusive as ruins. Um ADR sem consequência negativa listada geralmente não
descreve uma decisão real.

Status possíveis: `Proposto`, `Aceito`, `Substituído por ADR-XXX`, `Obsoleto`.

## Índice

| ADR | Título | Status |
| --- | --- | --- |
| [001](001-win32-boundary.md) | Fronteira única para acesso Win32 | Aceito |
| [002](002-build-profiles.md) | `BuildProfile` como unidade de compatibilidade | Aceito |
| [003](003-save-transactions.md) | Backup obrigatório em transação de save | Aceito |
| [004](004-offline-test-architecture.md) | Memória falsa em vez de mocks para testes offline | Aceito |
| [005](005-frontend-state-model.md) | Estado do trainer como união discriminada | Aceito |
