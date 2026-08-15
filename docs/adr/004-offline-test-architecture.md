# ADR-004 — Memória falsa em vez de mocks para testes offline

**Status:** Aceito · **Data:** 2026-08-15 · **SPEC:** SPEC-006

## Contexto

Os testes anteriores exercitavam funcionalidade real, mas quase todos exigiam o
Deep Rock Galactic aberto, na build exata, com o personagem carregado na Space
Rig. Isso torna impossível ter uma suíte de regressão no CI — e, na prática,
significa que um refactor só é validado quando alguém lembra de abrir o jogo.

O problema não é falta de testes: é que o único ambiente onde eles rodam é o
mais caro e menos reprodutível possível.

## Decisão

Três níveis, com a linha divisória no que exige o processo real:

1. **`unit`** — funções puras, validações, catálogos.
2. **`integration-offline`** — módulos integrados sobre uma memória falsa.
3. **`live-game`** — atrás da feature `live-tests`, fora do `cargo test` padrão.

Para o nível 2, a escolha central: **`FakeMemory` é uma memória de verdade**, um
mapa de regiões sobre buffers, com as mesmas validações do backend Win32
(endereço mínimo, teto de transferência, leitura parcial como erro). Não é um
mock de expectativas.

Sobre ela, `TestWorld` monta uma `FNamePool` e uma `GUObjectArray` sintéticas —
usando **os offsets do perfil de produção** — e `SaveWorld` monta um
`FSDSaveGame` plausível.

Chamadas nativas não são simuladas: `FakeMemory` registra a tentativa e devolve
um exit code configurado.

## Alternativas consideradas

**Mocks com expectativas (`mockall` ou similar).** Rejeitada: um mock que
responde `Ok(0x1234)` a `read_u64` testa que o código chamou `read_u64`, não que
ele lê a estrutura certa. Os bugs reais deste projeto são de layout e de
travessia — precisamente o que um mock esconde.

**Fixtures de dump de memória real do jogo.** Rejeitada: dumps contêm dados de
save de quem gerou, são grandes, e não podem ser publicados. Um mundo sintético
é anonimizado por construção e legível no diff.

**Simular a execução das rotinas nativas.** Rejeitada: simular o que
`Cheat_UnlockAllWeapons` faz seria reimplementar o jogo — e o teste passaria a
validar a simulação. Registrar a tentativa responde à pergunta que importa: *a
rotina certa seria chamada, e ela **não** é chamada quando a assinatura
diverge?*

## Consequências

**Boas**

- `cargo test` roda em máquina que nunca teve o jogo instalado, o que torna o
  gate de CI possível.
- Como o mundo de teste usa os offsets do perfil real, **um offset errado no
  perfil quebra o teste** em vez de passar despercebido.
- Cenários de erro difíceis de reproduzir com o jogo aberto — índice interno
  divergente, GUID duplicado, ciclo na cadeia de classes — viram testes de três
  linhas.

**Ruins**

- Os tipos de infraestrutura ficaram genéricos sobre o backend de memória. É
  ruído nas assinaturas.
- O mundo sintético precisa ser mantido junto com o perfil: adicionar um offset
  novo às vezes exige ensinar o `SaveWorld` a montá-lo.
- Testes offline não provam que os offsets estão **corretos** para a build, só
  que são internamente coerentes. Os live tests continuam necessários — e é por
  isso que eles não foram removidos, apenas isolados.
