# ADR-003 — Backup obrigatório em transação de save

**Status:** Aceito · **Data:** 2026-08-15 · **SPEC:** SPEC-020

## Contexto

Operações como unlock de armas, promoção de classes e adição de recursos gravam
no arquivo de save. Elas são irreversíveis do ponto de vista do jogo: não há
"desfazer" depois que o `SaveToDisk` roda.

O código anterior já fazia backup antes dessas operações — era um dos pontos
fortes do projeto. Mas o backup era uma chamada de função que cada operação
lembrava de fazer, no lugar certo da sequência. Uma operação nova poderia
esquecer, ou fazer na ordem errada, e nada quebraria até alguém perder progresso.

## Decisão

Uma `SaveTransaction` centraliza o pipeline. Nenhum código de domínio escreve no
save sem abrir uma.

```text
localizar saves → validar → backup versionado → mutar → verificar → reportar
```

A transação é obtida via `session.begin_save_transaction(operação)`, e o backup
já aconteceu quando ela existe. A partir daí:

- `guard` propaga qualquer erro **preservando o código** e anexando o caminho de
  restauração;
- `verify` transforma uma verificação falha em `WRITE_VERIFICATION_FAILED` com o
  caminho;
- `ensure_same_save` aborta se o save ativo trocou no meio.

Backups nunca se sobrescrevem: um sufixo é adicionado quando o nome colide.

## Alternativas consideradas

**Manter a chamada de backup em cada operação.** Rejeitada: é a situação
anterior. Funciona, mas a garantia depende de lembrar.

**Snapshot em memória com rollback automático.** Rejeitada: o save é gravado
pela rotina nativa do próprio jogo, dentro do processo dele. Não há ponto onde o
trainer possa desfazer de forma confiável depois do `SaveToDisk`. Um rollback
que às vezes funciona é pior que um backup que sempre existe.

**Remoção automática de backups antigos.** Rejeitada. Discutida a sério, porque
os backups acumulam. Mas apagar automaticamente a rede de segurança do usuário
troca alguns megabytes por risco de perda irreversível. A política é: nunca
remover, informar quantos existem, documentar a limpeza manual.

## Consequências

**Boas**

- É impossível escrever uma operação permanente nova sem backup: o tipo que dá
  acesso ao pipeline só existe depois que o backup foi criado.
- Toda mensagem de erro pós-backup diz onde estão os artefatos — sem isso, o
  usuário sabe que algo falhou mas não como voltar.
- Colisão de timestamp virou teste, não descoberta em produção.

**Ruins**

- Backups acumulam indefinidamente. Mitigado pelo relato de contagem e pela
  documentação em `docs/save-restore.md`, mas o disco é do usuário.
- Cada mutação paga o custo de copiar os saves, mesmo quando falha logo depois.
  Aceito: a alternativa é decidir quando o backup "não vale a pena", que é
  exatamente o julgamento que se quer eliminar.
- A restauração continua sendo manual. Automatizá-la exigiria mexer nos saves do
  usuário sem supervisão, o que contradiz o propósito do backup.
