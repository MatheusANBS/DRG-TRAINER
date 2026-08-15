# ADR-002 — `BuildProfile` como unidade de compatibilidade

**Status:** Aceito · **Data:** 2026-08-15 · **SPEC:** SPEC-002, SPEC-003

## Contexto

Hash do executável, offsets, assinaturas de rotinas nativas e contagens de
catálogo descrevem, juntos, uma versão específica do Deep Rock Galactic. No
código anterior eles viviam como constantes de módulo, misturados com a lógica
genérica de leitura e travessia.

Isso funciona para uma build. Para a segunda, exige editar a lógica genérica —
e, pior, torna possível que um offset da build antiga seja usado sem que ninguém
perceba, porque não existe um lugar onde "a build mudou" seja uma pergunta com
resposta.

## Decisão

Um `BuildProfile` imutável e `'static` por build suportada, em
`src-tauri/src/build_profiles/profiles/`. O perfil carrega SHA-256 esperado,
offsets, assinaturas nativas, catálogo e **capacidades verificadas**.

O runtime resolve o perfil pela hash do executável **antes** de expor qualquer
recurso dependente de memória. O registry devolve `Supported`, `Unverified` ou
`Unsupported { sha256 }`.

Dois mecanismos separados de gate:

1. **`ProfileStatus`** — só `Supported` libera operações. `Draft` e `Verified`
   existem para o onboarding de uma build nova.
2. **`Capabilities`** — granular por operação. Uma build pode ter créditos
   verificados e esquemas ainda não; a UI desabilita só o que falta.

## Alternativas consideradas

**Um arquivo de configuração externo (JSON/TOML) com os offsets.** Rejeitada:
transforma dado crítico de segurança em algo editável por quem baixou o binário.
Um offset errado num arquivo de texto escreve em endereço arbitrário do jogo.
Compilar os perfis mantém a distribuição íntegra e ainda dá checagem de tipo.

**Detectar offsets dinamicamente por scan de assinatura.** Rejeitada por ora:
resolve o problema errado. O scan encontra rotinas, mas os offsets de campos de
save continuam precisando de verificação humana — e um scan que "quase acerta"
é mais perigoso que uma recusa.

**Só a hash, sem `Capabilities`.** Rejeitada: obrigaria a validar tudo de uma vez
antes de suportar qualquer coisa numa build nova. O gate granular permite
liberar créditos no dia 1 e esquemas na semana seguinte, sem mentir para o
usuário sobre o que foi verificado.

## Consequências

**Boas**

- Suportar uma build nova é criar um arquivo e registrá-lo em `ALL`.
- "A build mudou" tem resposta explícita e visível na interface.
- Capacidade não verificada é indistinguível, para o usuário, de capacidade
  inexistente — que é exatamente o comportamento seguro.
- Os testes de consistência de perfil pegam hash malformada, GUID duplicado e
  assinatura vazia antes do merge.

**Ruins**

- Perfis se duplicam entre builds próximas. Mitigado por tipos compartilhados,
  mas a duplicação é real e deliberada: herdar offsets é justamente o erro que
  se quer impedir.
- Cada build nova exige recompilar e republicar. Aceito: a alternativa é
  configuração externa, rejeitada acima.
- O passo "zere as capacidades ao copiar um perfil" é manual e crítico. Mitigado
  pelo checklist em `docs/build-support.md`, mas depende de disciplina.
