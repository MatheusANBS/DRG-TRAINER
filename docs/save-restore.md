# Backup e restauração de save

O que o trainer preserva antes de cada mutação permanente, e como voltar atrás
(SPEC-020).

## O que é copiado

Antes de qualquer operação que grave no save em disco, o trainer copia:

- o save principal (`*_Player.sav`);
- os saves de slot (`*_Player_Slot_*.sav`).

Backups externos do próprio jogo (`*_ExternalBackup_*`) são ignorados, para não
duplicar dados que o jogo já gerencia.

Se nenhum save for encontrado, a operação **falha antes de mudar qualquer
coisa** — um backup vazio daria falsa sensação de segurança.

## Onde ficam

```text
<jogo>\FSD\Saved\SaveGames\DRGTrainerBackups\<operação>-<timestamp>\
```

Exemplo:

```text
FSD\Saved\SaveGames\DRGTrainerBackups\unlock-all-1755264000\
FSD\Saved\SaveGames\DRGTrainerBackups\promote-all-classes-1755264012\
```

Se duas operações caírem no mesmo segundo, a segunda recebe sufixo
(`-2`, `-3`, …). **Um backup nunca sobrescreve outro em silêncio.**

O caminho exato aparece na notificação de sucesso e em qualquer mensagem de erro
posterior ao backup.

## Como restaurar

1. **Feche o Deep Rock Galactic.** Restaurar com o jogo aberto faz o save em
   memória sobrescrever o arquivo restaurado ao sair.
2. Abra o diretório de backup da operação que quer desfazer.
3. Copie os arquivos `.sav` de volta para `FSD\Saved\SaveGames\`,
   sobrescrevendo.
4. Abra o jogo e confira o progresso antes de qualquer nova operação.

```powershell
# Feche o jogo antes de rodar.
$saves  = "<jogo>\FSD\Saved\SaveGames"
$backup = "$saves\DRGTrainerBackups\unlock-all-1755264000"

# Confira o que será restaurado.
Get-ChildItem $backup

# Restaure.
Copy-Item "$backup\*.sav" $saves -Force
```

## Política de retenção

**Backups nunca são removidos automaticamente.**

Eles são a rede de segurança do usuário; apagar automaticamente trocaria espaço
em disco por risco de perda de progresso. Cada backup tem alguns megabytes, e o
trainer informa quantos existem no resultado de cada operação
(`backup.totalBackups`).

A limpeza é manual e deliberada:

```powershell
# Liste do mais antigo para o mais novo antes de decidir.
Get-ChildItem "$saves\DRGTrainerBackups" -Directory | Sort-Object CreationTime

# Remova apenas o que você já revisou.
Remove-Item "$saves\DRGTrainerBackups\<operação>-<timestamp>" -Recurse
```

## Quando uma operação falha

Toda falha ocorrida **após** o backup inclui o caminho de restauração na
mensagem. Códigos que exigem atenção especial:

| Código | Significado | O que fazer |
| --- | --- | --- |
| `WRITE_VERIFICATION_FAILED` | A escrita não foi confirmada na releitura | Restaure o backup antes de tentar de novo |
| `SAVE_STATE_CHANGED` | O save ativo mudou durante a operação | Feche o jogo, restaure, e refaça sem trocar de personagem |
| `NATIVE_CALL_FAILED` / `NATIVE_CALL_TIMEOUT` | A rotina do jogo não concluiu | Confira o progresso; restaure se estiver inconsistente |
| `BACKUP_FAILED` | O backup não foi criado | Nada foi alterado; corrija o caminho dos saves |

`BACKUP_FAILED` é o único caso em que é garantido que **nada** mudou: a operação
para antes da mutação.
