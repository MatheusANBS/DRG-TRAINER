# Testes

Três níveis, com fronteiras claras (SPEC-006). A regra que organiza tudo:
**`cargo test` e `npm test` precisam passar em uma máquina que nunca teve o
Deep Rock Galactic instalado.**

## Níveis

| Nível | O que exercita | Roda no CI | Como rodar |
| --- | --- | --- | --- |
| `unit` | Parsers, validações, catálogos, cálculo de endereço, máquinas de estado | Sim | `cargo test` / `npm test` |
| `integration-offline` | Módulos integrados sobre memória falsa e sobre a API pública do crate | Sim | `cargo test` / `npm test` |
| `live-game` | Processo real do DRG, na build suportada | Não | `cargo test --features live-tests` |

## Backend

### Offline

```powershell
cd src-tauri
cargo test --locked
```

O que sustenta o nível offline:

- **`FakeMemory`** (`infrastructure/process/fake.rs`) — memória de processo
  simulada por regiões sobre buffers reais, com as mesmas validações do backend
  Win32: endereço mínimo, teto de transferência, leitura/escrita parcial como
  erro. Não é um mock de expectativas; é memória de verdade, em processo.
- **`TestWorld`** (`infrastructure/unreal/testing.rs`) — `FNamePool` e
  `GUObjectArray` sintéticas montadas com **os offsets do perfil de produção**.
  Isso é o ponto: um offset errado no perfil quebra o teste, em vez de passar
  despercebido até alguém abrir o jogo.
- **`SaveWorld`** (`domain/testing.rs`) — um `FSDSaveGame` plausível com mapa de
  recursos, `CharacterSaves` e arrays de progressão.

Chamadas nativas **não são simuladas**. `FakeMemory` registra a tentativa e
devolve o exit code configurado, para que um teste possa afirmar *que* a rotina
seria chamada — e, principalmente, que ela **não** é chamada quando a assinatura
diverge.

### Integração pela API pública

`src-tauri/tests/offline_contracts.rs` roda contra o crate como um consumidor
externo. É onde ficam travados os contratos de que o frontend e o processo de
release dependem: resolução de build, status operacional, serialização de erro.

### Live-game

```powershell
cd src-tauri
cargo test --features live-tests -- --nocapture --test-threads=1
```

Requisitos:

- DRG aberto na build suportada;
- personagem carregado na Space Rig;
- arma equipada (para os testes de arma);
- **save descartável**.

`--test-threads=1` não é opcional: os testes compartilham o processo do jogo e
os toggles globais.

Os live tests resolvem alvos e conferem assinaturas, mas **não executam
mutações permanentes**. Unlock, promoção e adição de recursos continuam sendo
verificação manual — automatizar uma escrita irreversível em um save real vale
menos do que o risco.

## Frontend

```powershell
npm test          # watch
npm test -- --run # uma passada, como no CI
```

Os testes consultam por papel, rótulo e estado — nunca por classe CSS. É o que
permite refinar o visual (SPEC-014/017) sem quebrar a suíte.

Cobertura por comportamento, não por percentual:

| Comportamento | Arquivo |
| --- | --- |
| Estado derivado do backend, bloqueio por capacidade | `src/state/trainer-state.test.ts` |
| Tradução de erro e normalização da fronteira | `src/lib/errors.test.ts` |
| Formatação de arma e captura de hotkey | `src/lib/weapon-format.test.ts` |
| Status operacional, gating, confirmação, duplo clique, erro, navegação | `src/App.test.tsx` |

Os mocks da fronteira Tauri ficam centralizados em
`src/test/trainer-api-mock.ts`. Nenhum teste mocka `@tauri-apps/api`
diretamente: a fronteira real é `services/trainer-api`, e é ela que precisa ser
substituída para o teste refletir o que a aplicação realmente chama.

## Escrevendo um teste novo

1. Ele precisa do jogo aberto? Se sim, ele é `live-game` e vai atrás da feature.
2. Precisa de memória? Use `FakeMemory`/`TestWorld`/`SaveWorld`, não um mock.
3. Está testando implementação ou comportamento? Prefira o segundo — no
   frontend, consulte por papel e texto.
4. Cubra o caminho de erro, não só o feliz. Metade das garantias deste projeto
   é sobre recusar operações.
