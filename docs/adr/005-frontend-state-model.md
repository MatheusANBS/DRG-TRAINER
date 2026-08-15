# ADR-005 — Estado do trainer como união discriminada

**Status:** Aceito · **Data:** 2026-08-15 · **SPEC:** SPEC-012, SPEC-013, SPEC-019

## Contexto

`App.tsx` concentrava polling, estado, ações e hotkeys em ~600 linhas, com mais
de trinta `useState`. O estado de conexão vivia em uma string (`"connecting" |
"connected" | "waiting" | "error"`) somada a um `snapshot` possivelmente nulo e
a um booleano `buildValid` embutido no snapshot.

Cada botão recalculava `disabled` a partir dessa combinação — na prática,
`!snapshot || busy`. Isso é suficiente enquanto existe uma build suportada e
nenhuma capacidade condicional. Com `BuildProfile` (ver
[ADR-002](002-build-profiles.md)), passa a existir um estado que a UI precisa
representar de verdade: *anexado, mas não é seguro operar*.

## Decisão

Um estado derivado, discriminado por `kind`:

```ts
type TrainerConnection =
  | { kind: "connecting" }
  | { kind: "detached"; expected: string[] }
  | { kind: "unsupported"; pid; executable; sha256 }
  | { kind: "unverified"; pid; executable; profileId; sha256 }
  | { kind: "attached"; pid; executable; profileId; displayName; sha256 }
  | { kind: "error"; error: TrainerError };
```

Consequências diretas dessa escolha:

- **`blockedReason(state, capability)` é a única fonte de `disabled`.** Ela
  devolve o motivo em texto, o que força cada bloqueio a ser explicável — um
  botão desabilitado sem explicação vira suporte.
- **Build não suportada é um estado, não um erro.** O status bar diz `GAME
  ATTACHED` + `BUILD UNSUPPORTED`, que é a verdade; um banner vermelho de erro
  não seria.
- **"Aguardando o jogo" não é falha.** `PROCESS_NOT_FOUND` vira `detached`, não
  `error`.

A orquestração saiu de `App.tsx` para hooks por caso de uso, `invoke` ficou
restrito a `services/trainer-api.ts`, e o polling passou a agendar o próximo
ciclo só quando o anterior termina.

## Alternativas consideradas

**Uma biblioteca de estado (Zustand, Redux).** Rejeitada: o estado é derivado de
um único comando do backend. Uma store global acrescenta indireção sem resolver
o problema, que era representação, não propagação.

**Manter booleanos, adicionando `buildSupported` e `capabilities`.** Rejeitada:
é o caminho que produziu o problema. Com cinco booleanos, dezesseis das trinta e
duas combinações são impossíveis, e nada impede a UI de renderizar uma delas.

**Uma máquina de estados formal (XState).** Rejeitada: não há transições
complexas nem efeitos por transição. O estado é uma projeção pura do status do
backend; uma união discriminada com uma função de derivação expressa isso
melhor, e sem dependência.

## Consequências

**Boas**

- Um estado novo é uma variante nova, e o `switch` exaustivo mostra onde tratá-la.
- `App.tsx` caiu para ~150 linhas e não faz nenhuma chamada ao backend.
- Todo bloqueio tem explicação, porque a função que bloqueia devolve texto.
- Testar "unsupported build desabilita tudo" virou uma asserção sobre a função
  de derivação, sem render.

**Ruins**

- Duas representações do mesmo domínio: os DTOs do backend e o estado da UI, com
  uma função de tradução entre eles. Deliberado — a UI precisa de estados que o
  backend não tem (`connecting`) — mas é código a manter.
- Prop drilling de `state` para os hooks de ação. Aceitável nesta escala;
  contexto seria a saída se crescer.
- O contrato de tipos entre `dto.rs` e `types/trainer.ts` é mantido à mão. Os
  testes de contrato em `offline_contracts.rs` cobrem a serialização, mas a
  correspondência de campo depende de revisão.
