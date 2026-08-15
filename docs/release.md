# Release

Como uma versão é produzida, verificada e distribuída (SPEC-011 / SPEC-022).

## Pipeline

`.github/workflows/release.yml` dispara em tags `v*` e tem dois jobs:

1. **`gates`** — exatamente os mesmos gates do PR: lint e testes do frontend,
   `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `npm audit`. Um
   artefato nunca sai de um commit que não passaria em um pull request.
2. **`publish`** — só roda se `gates` passar. Compila, gera o manifesto, assina
   (quando houver certificado), gera checksums e anexa à release.

Permissões seguem o mínimo: o workflow declara `contents: read`, e apenas o job
`publish` eleva para `contents: write`, exclusivamente para anexar artefatos.

## Artefatos

| Arquivo | Origem |
| --- | --- |
| `DRG-Runtime-Trainer.exe` | `src-tauri/target/release/drg-credits.exe` |
| `build-manifest.json` | Gerado no CI a partir do commit e dos perfis |
| `SHA256SUMS.txt` | Gerado no CI, nunca na máquina de quem publica |

### Manifesto de build

```json
{
  "product": "DRG Runtime Trainer",
  "tag": "v0.2.0",
  "commit": "…",
  "builtAt": "2026-08-15T12:00:00Z",
  "workflowRun": "…",
  "signed": false,
  "supportedBuilds": [
    { "profileId": "fsd-8e22e371", "executableSha256": "8e22e371…" }
  ]
}
```

O manifesto responde à pergunta que o checksum sozinho não responde: *este
binário suporta a minha build do jogo?*

## Verificação pelo usuário

```powershell
Get-FileHash .\DRG-Runtime-Trainer.exe -Algorithm SHA256
Get-Content .\SHA256SUMS.txt
```

Os dois valores precisam bater, ignorando maiúsculas. Se não baterem, não
execute o arquivo.

Para conferir a build suportada:

```powershell
Get-Content .\build-manifest.json | ConvertFrom-Json | Select-Object -ExpandProperty supportedBuilds
Get-FileHash "<caminho>\FSD\Binaries\Win64\FSD-Win64-Shipping.exe" -Algorithm SHA256
```

## Assinatura de código

O estágio de assinatura Authenticode já existe no pipeline e é condicional: ele
roda quando `WINDOWS_CERT_BASE64` está configurado nos secrets do repositório.
Ligar a assinatura é adicionar dois secrets, não redesenhar o pipeline.

- `WINDOWS_CERT_BASE64` — o `.pfx` em base64
- `WINDOWS_CERT_PASSWORD` — a senha do `.pfx`

Regras:

- nenhuma chave, certificado ou senha vive no repositório;
- os secrets ficam apenas no ambiente de release;
- o `.pfx` é escrito em `RUNNER_TEMP` e removido no mesmo passo;
- o manifesto passa a registrar `"signed": true`.

Enquanto não houver certificado, o SmartScreen exibirá aviso na primeira
execução. Isso é esperado e está documentado no README.

## Checklist de release

Ver [release-checklist.md](release-checklist.md). Uma release não sai sem o
checklist preenchido.

## SBOM e provenance

Fora do escopo atual. Quando a distribuição crescer, o ponto natural de entrada
é um passo adicional em `publish` gerando CycloneDX ou SPDX a partir de
`Cargo.lock` e `package-lock.json`, anexado como artefato ao lado do manifesto.
