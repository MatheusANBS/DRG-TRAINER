# DRG Runtime Trainer

Trainer desktop para **Deep Rock Galactic**, desenvolvido em Rust/Tauri com interface React. O aplicativo acompanha objetos do runtime do Unreal Engine e reúne ferramentas de inventário, progressão e armas em uma interface compacta.

![Visão geral do DRG Runtime Trainer](docs/screenshots/trainer-overview.png)

_As capturas usam o modo de demonstração do frontend; valores e endereços exibidos são fictícios._

## Recursos

### Player

- Define Driller, Engineer, Gunner e Scout no nível 25.
- Promove as quatro classes uma vez usando a progressão nativa do jogo.
- Libera todos os perks.

### Inventory

- Lê e altera créditos com verificação após a escrita.
- Exibe os 14 recursos persistentes do inventário.
- Adiciona uma quantidade a um recurso específico ou a todos os recursos.

![Gerenciamento de inventário](docs/screenshots/trainer-inventory.png)

### Weapons

- Infinite Magazine acompanha a arma equipada e mantém o pente cheio.
- Weapon Damage acompanha os componentes de dano da arma ativa e mantém o valor em `9999`.
- Libera armas, modificações de equipamento, overclocks e cosméticos.
- Permite configurar hotkeys globais para recursos em tempo real.

![Ferramentas de armas](docs/screenshots/trainer-weapons.png)

## Compatibilidade e segurança

O trainer resolve um perfil de build pelo SHA-256 do `FSD-Win64-Shipping.exe` antes de acessar qualquer offset ou executar qualquer rotina nativa. O status operacional aparece no cabeçalho:

```text
● GAME ATTACHED    ✓ BUILD VERIFIED
FSD-Win64-Shipping.exe · PID 26248 · profile fsd-8e22e371
```

Se a build mudar, o cabeçalho passa a mostrar `BUILD UNSUPPORTED` e todas as ações dependentes de memória ficam desabilitadas até que um perfil seja verificado — ver [docs/build-support.md](docs/build-support.md).

Cada capacidade é verificada por build. Uma ação não verificada no perfil ativo permanece desabilitada e explica o motivo, em vez de tentar e falhar.

Operações permanentes criam uma cópia dos saves antes de escrever:

```text
FSD/Saved/SaveGames/DRGTrainerBackups/<operação>-<timestamp>/
```

Backups nunca se sobrescrevem e nunca são removidos automaticamente. O procedimento de restauração está em [docs/save-restore.md](docs/save-restore.md).

O objeto ativo de save e a arma equipada são resolvidos dinamicamente e revalidados antes do acesso. Ainda assim, este projeto modifica memória e progresso salvo: mantenha backups e prefira uso solo ou em sessões privadas.

## Instalação

1. Baixe `DRG-Runtime-Trainer.exe` na página de Releases.
2. Abra o Deep Rock Galactic e entre na Space Rig.
3. Execute o trainer.

O executável é portátil e não possui instalador. Como não há assinatura de código, o Windows pode exibir o aviso do SmartScreen na primeira execução.

## Desenvolvimento

Pré-requisitos:

- Windows 10 ou 11 x64
- Node.js 20 ou mais recente
- Rust stable com target MSVC
- Microsoft Edge WebView2 Runtime

```powershell
npm install
npm run tauri dev
```

Para trabalho puramente visual, `npm run dev` sobe o frontend no browser com um backend simulado — sem o jogo e sem Tauri.

Gates de qualidade (os mesmos do CI):

```powershell
npm run lint
npm test -- --run
npm run build
```

```powershell
cd src-tauri
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

`cargo test` passa com o jogo fechado. Testes que exigem o DRG em execução ficam atrás da feature `live-tests` — ver [docs/testing.md](docs/testing.md).

Build de release:

```powershell
npm run tauri build
```

O executável será gerado em `src-tauri/target/release/drg-credits.exe`.

## Estrutura

```text
src/
  App.tsx                   composition root
  hooks/                    conexão, polling, ações e hotkeys
  services/                 única fronteira invoke
  state/                    estado observável do trainer
  components/trainer/       seções e primitivas da interface
  components/ui/            primitivas shadcn/Radix
  lib/                      funções puras (erros, formatação)
  types/                    contratos com o backend e semântica de ação
src-tauri/src/
  app/                      comandos Tauri e atalhos globais
  domain/                   regras do DRG (inventário, progressão, jogador, armas)
  infrastructure/           Win32, runtime Unreal e transações de save
  build_profiles/           dados específicos de cada build do jogo
  shared/                   erros tipados e limites de segurança
```

## Documentação

| Documento | Para quê |
| --- | --- |
| [docs/architecture.md](docs/architecture.md) | Mapa das camadas e invariantes |
| [docs/build-support.md](docs/build-support.md) | Suportar uma build nova do jogo |
| [docs/testing.md](docs/testing.md) | Os três níveis de teste |
| [docs/frontend-design.md](docs/frontend-design.md) | Contrato visual e tokens |
| [docs/release.md](docs/release.md) | Pipeline, checksums e assinatura |
| [docs/save-restore.md](docs/save-restore.md) | Backup e restauração |
| [docs/adr/](docs/adr/) | Decisões arquiteturais duráveis |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Como contribuir |

## Aviso

Projeto independente, sem associação com Ghost Ship Games ou Coffee Stain Publishing. Deep Rock Galactic e seus elementos pertencem aos respectivos titulares.

## Segurança

Relatórios de segurança devem seguir as orientações de [SECURITY.md](SECURITY.md). Não publique saves, tokens ou dumps completos de memória em issues.
