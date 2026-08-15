# ADR-001 — Fronteira única para acesso Win32

**Status:** Aceito · **Data:** 2026-08-15 · **SPEC:** SPEC-001, SPEC-004

## Contexto

O trainer lê e escreve na memória de outro processo, aloca páginas executáveis e
cria threads remotas. Um erro aqui não gera um bug de interface: corrompe o
progresso de quem usa, ou derruba o jogo.

O código anterior já tinha boas garantias — handles RAII, verificação de bytes,
separação entre leitura e escrita — mas elas viviam por convenção. Nada impedia
que um módulo de domínio novo chamasse `WriteProcessMemory` diretamente e
esquecesse uma delas.

## Decisão

Todo acesso Win32 a processo e memória fica confinado a
`src-tauri/src/infrastructure/process/`. O domínio conversa apenas com os traits
`MemoryReader`, `MemoryWriter` e `NativeCaller`.

Os invariantes passam a ser propriedades do tipo, não da disciplina:

- `ProcessMemory::open` recebe `Access::ReadOnly` ou `Access::ReadWrite`;
  escrever com um handle read-only é `ErrorCode::ReadOnlyHandle`, não UB.
- `OwnedHandle` não expõe API de vazamento.
- `validate_range` roda em toda leitura e escrita.
- `write_i32_verified` relê e confere; uma escrita não confirmada é erro.
- `call_verified` valida o prefixo de código antes de qualquer chamada nativa.

## Alternativas consideradas

**Manter por convenção, com revisão de código.** Rejeitada: funciona enquanto o
projeto tem um mantenedor com o contexto todo na cabeça. O objetivo declarado é
sobreviver a dezenas de atualizações do jogo.

**Marcar as funções perigosas como `unsafe` e parar por aí.** Rejeitada: quase
todo o código já é `unsafe` por natureza; o marcador perde o poder de sinalizar.
A garantia útil não é "isto é perigoso", é "isto verifica".

**Uma crate separada para a camada de processo.** Rejeitada por ora: acrescenta
cerimônia de workspace sem acrescentar garantia — a fronteira de módulo já
impede a chamada direta, e o `pub(crate)` faz o resto.

## Consequências

**Boas**

- Auditar a segurança de memória significa ler um diretório.
- Substituir o backend real por um falso nos testes vira uma troca de tipo
  genérico (ver [ADR-004](004-offline-test-architecture.md)).
- Um invariante novo é adicionado em um lugar e vale para todo o domínio.

**Ruins**

- Uma camada a mais entre o domínio e a syscall: rastrear um erro exige pular
  por um trait. Mitigado mantendo os wrappers finos e sem lógica.
- Os tipos de domínio ficaram genéricos sobre `M: MemoryReader`, o que aparece
  nas assinaturas. É o preço de testar offline.
- Um caso de uso legítimo que precise de uma API Win32 nova exige estender a
  infraestrutura em vez de chamar direto. Isso é intencional.
