# Contrato visual do frontend

O que a interface promete e o que não pode mudar sem decisão explícita
(SPEC-014 a SPEC-019).

## Princípio

A UI é **densa por escolha**: tudo que importa cabe em 940x660 sem rolagem, e o
usuário opera com o jogo aberto atrás. Densidade não é desculpa para
ilegibilidade — daí o piso tipográfico e os alvos mínimos abaixo.

## Tokens

Definidos em `src/styles.css`, sob `:root`. Use os tokens; um valor mágico
repetido é o começo de uma escala paralela.

### Tipografia

| Token | Valor | Uso |
| --- | --- | --- |
| `--text-meta-sm` | 9px | Rótulos secundários (`ADDRESS`, `CLASS`) |
| `--text-meta` | 10px | Metadados, pills, hotkeys, botões |
| `--text-section` | 11px | Títulos de seção |
| `--text-option` | 12px | Nome da opção — o texto mais lido da tela |
| `--text-value` | 13px | Valor principal (créditos) |

**Nenhum texto funcional abaixo de 9px.** O piso anterior era 7px, ilegível em
1080p com scaling do Windows.

### Espaçamento

`--space-1` a `--space-8`, em múltiplos de 2px (2, 4, 6, 8, 10, 12, 14, 18).

### Controles

| Token | Valor |
| --- | --- |
| `--control-height` | 26px |
| `--row-min-height` | 50px |
| `--radius-control` | 4px |
| `--radius-surface` | 6px |

### Cor

Paleta: carvão, cinza quente, âmbar, verde.

| Papel | Token | Regra |
| --- | --- | --- |
| Sucesso / ativo | `--state-ok` (verde) | Estado ligado, operação concluída |
| Atenção / permanente | `--state-warning` (âmbar) | Mutação irreversível, build não verificada |
| Erro | `--state-error` (vermelho) | **Somente** falha real |
| Neutro | `--state-idle` | Desconhecido, aguardando |

**Vermelho é reservado a erro.** Mutação permanente usa âmbar reforçado: ela é
irreversível, não destrutiva, e a diferença importa.

## Semântica de ação (SPEC-015)

Toda ação declara seu impacto em `src/types/actions.ts`:

| Semântica | Badge | Confirmação | Exemplo |
| --- | --- | --- | --- |
| `runtime` | — | Não | Infinite Magazine, Weapon Damage |
| `standard` | — | Sim | Editar créditos |
| `persistent` | `PERMANENT` | Sim | Unlock All Weapons, Promote All Classes |

O componente `ActionControl` recebe a semântica em vez de deduzi-la do texto.
Adicionar uma ação sem declarar semântica é um erro de tipo.

Badges aparecem apenas quando acrescentam informação: `runtime` e `standard` não
têm badge, porque "esta ação não é permanente" não é notícia.

## Status operacional (SPEC-016)

O cabeçalho responde, em texto, às duas perguntas que precedem qualquer ação:

```text
● GAME ATTACHED    ✓ BUILD VERIFIED
FSD-Win64-Shipping.exe · PID 26248 · profile fsd-8e22e371
```

```text
○ GAME ATTACHED    ! BUILD UNSUPPORTED
⚠ Memory-dependent actions disabled until a build profile matches.
```

Regras:

- estado é **texto**; cor e ícone reforçam, não substituem;
- build não suportada nunca depende de tooltip;
- tooltip carrega detalhe, jamais informação essencial;
- todo controle desabilitado explica o motivo em tooltip.

## Estados assíncronos (SPEC-019)

Cinco fases por ação: `idle`, `pending`, `success`, `error`, `blocked`.

- `useAsyncAction` impede disparo duplicado por construção — duplo clique, Enter
  repetido e hotkey simultânea caem no mesmo guard;
- `blocked` vem de `blockedReason` no estado central; nenhuma tela recalcula
  `disabled`;
- resultado vai para o feed (`useActionFeed`), que some sozinho;
- estado persistente importante — conexão, build, toggles — vive no status bar e
  nas seções, **nunca** só em um toast.

## Responsividade (SPEC-018)

O contrato da janela Tauri é `minWidth: 760`, `minHeight: 590`. O breakpoint
compacto fica em **820px** — a primeira largura em que duas colunas apertam com
a tipografia atual.

O breakpoint anterior era 700px, **abaixo** do mínimo da janela: o
comportamento responsivo nunca era alcançado no uso real.

Tamanhos verificados, sem overflow horizontal:

| Tamanho | Layout | Observação |
| --- | --- | --- |
| 760x590 | Uma coluna | Mínimo da janela |
| 820x620 | Uma coluna | Último tamanho compacto |
| 821x620 | Duas colunas | Primeiro tamanho amplo |
| 940x660 | Duas colunas | Padrão; cabe sem rolagem |

Como o Tauri dimensiona a janela em pixels lógicos, 125% e 150% de scaling do
Windows não reduzem o viewport CSS — a janela fica maior fisicamente e o layout
permanece o mesmo.

## Identidade (SPEC-017)

Linguagem: painel industrial + ferramenta de engenharia + DRG.

O que existe:

- marca `DRG` monoespaçada em moldura âmbar;
- wordmark discreto ao lado (some abaixo de 820px);
- faixa de aviso hachurada de 2px sob o cabeçalho;
- gradiente sutil no topo da superfície.

O que **não** entra: artwork grande, neon, glassmorphism, decoração que compita
com controles, ou qualquer asset protegido sem licença adequada.

Teste de aceitação: comparar antes/depois em 940x660. Se a decoração chamar mais
atenção que o botão mais próximo, ela perdeu.

## Acessibilidade

- foco sempre visível (`:focus-visible` com anel de 2px);
- estado disponível como texto para leitores de tela;
- `role="status"` no status bar, `role="alert"` em erro;
- `prefers-reduced-motion` desliga animações;
- rótulos acessíveis em todos os switches, botões de ícone e campos.
