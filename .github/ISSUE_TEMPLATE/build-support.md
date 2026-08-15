---
name: Nova build do jogo
about: O trainer mostra BUILD UNSUPPORTED após uma atualização do Deep Rock Galactic
title: "Suporte à build <sha256 curta>"
labels: build-support
---

## Hash do executável

```powershell
Get-FileHash "<caminho>\FSD\Binaries\Win64\FSD-Win64-Shipping.exe" -Algorithm SHA256
```

SHA-256: `_______`

O status bar do trainer também mostra o início da hash lida.

## Ambiente

- Versão do trainer:
- Origem do jogo (Steam / Xbox / outra):
- Data da atualização do jogo:

## O que o trainer mostra

Cole o texto do status bar (as duas linhas).

## Checklist de onboarding

Acompanhamento do trabalho — ver [docs/build-support.md](../../docs/build-support.md).

- [ ] Perfil criado como `Draft` com `Capabilities::NONE`
- [ ] Offsets do runtime Unreal conferidos
- [ ] Offsets de `FSDSaveGame` conferidos
- [ ] Assinaturas nativas conferidas nesta build
- [ ] Validações offline verdes
- [ ] Live tests executados com save descartável
- [ ] Mutações permanentes verificadas manualmente
- [ ] Capacidades habilitadas só onde houve verificação
- [ ] Perfil promovido a `Supported`
- [ ] Histórico de builds atualizado

> Não envie arquivos `.sav`, backups ou dumps de memória nesta issue.
