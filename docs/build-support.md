# Suporte a builds do DRG

Procedimento repetível para absorver uma atualização do jogo (SPEC-003). O
objetivo é que um patch vire trabalho de checklist, não investigação ad hoc.

## Estados de um perfil

| Estado | Significado | Habilita memória? |
| --- | --- | --- |
| `draft` | Offsets preenchidos, nada validado | Não |
| `verified` | Validações offline passaram, live tests pendentes | Não |
| `supported` | Live tests aprovados e evidências registradas | Sim |
| `deprecated` | Build antiga, mantida para diagnóstico | Não |

Somente perfis `supported` liberam operações dependentes de offset. Um binário
público deve conter apenas perfis `supported`; `draft` e `verified` existem para
o trabalho de onboarding e podem ser distribuídos apenas em builds de
desenvolvimento.

## Passo a passo

### 1. Identificar a build

```powershell
Get-FileHash "<caminho>\FSD\Binaries\Win64\FSD-Win64-Shipping.exe" -Algorithm SHA256
```

Abra o trainer: o status bar mostra `BUILD UNSUPPORTED` e a hash lida. Registre
a hash em minúsculas.

### 2. Duplicar o perfil anterior

```powershell
Copy-Item src-tauri/src/build_profiles/profiles/fsd_8e22e371.rs `
          src-tauri/src/build_profiles/profiles/fsd_<prefixo>.rs
```

No arquivo novo:

- troque `id`, `display_name` e `executable_sha256`;
- **coloque `status: ProfileStatus::Draft`**;
- **zere todas as capacidades**: `capabilities: Capabilities::NONE`.

Registre o módulo em `profiles/mod.rs` e adicione-o a `ALL`.

Zerar as capacidades é o passo que impede o pior cenário: um offset herdado da
build anterior escrevendo no lugar errado de um save real.

### 3. Atualizar offsets e assinaturas

Para cada área, confirme na build nova antes de habilitar a capacidade:

| Área | O que verificar | Capacidade |
| --- | --- | --- |
| Unreal | `GUObjectArray` RVA, `FNamePool` RVA, layout de `UObject`/`FProperty` | (base para tudo) |
| Save | offsets de `FSDSaveGame`, índices de classe e do save ativo | `credits` |
| Recursos | `Resources` map, `ResourceData.SavegameID`, `AddResource` | `resources` |
| Progressão | `OwnedPerks`, `PurchasedItemUpgrades`, `UnlockedItems`, `OwnedItems` | `perkUnlock`, `gearUnlock`, `weaponUnlock` |
| Esquemas | `SchematicSave`, `AllSchematics`, slot de vtable de `GrantReward` | `schematicUnlock` |
| Classes | `CharacterSaves`, GUIDs das quatro classes, `RetireCharacter` | `classLevel`, `promotion` |
| Armas | `ControllerPawn`, `PlayerInventory`, `ClipSize`, `ClipCount` | `infiniteMagazine`, `weaponDamage` |

Cada `NativeFunction` precisa de RVA **e** prefixo. Um prefixo copiado sem
conferir na build nova é pior do que nenhum: ele transforma uma checagem de
segurança em carimbo.

### 4. Validações offline

```powershell
cd src-tauri
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Os testes de consistência de perfil (`build_profiles/registry.rs`) verificam
hash bem formada, ids e GUIDs únicos, assinaturas não vazias e catálogo de
classes completo. Eles não sabem se o offset está *correto* — só se o perfil é
internamente coerente.

### 5. Live tests

Com o DRG aberto na build nova, na Space Rig, com personagem carregado:

```powershell
cd src-tauri
cargo test --features live-tests -- --nocapture --test-threads=1
```

Comece com um save descartável. Os live tests não executam mutações
permanentes: eles resolvem alvos e conferem assinaturas. As mutações precisam de
verificação manual, uma por vez, conferindo o resultado no jogo.

### 6. Promover capacidade por capacidade

Habilite em `capabilities` apenas o que foi verificado nesta build. Uma
capacidade não verificada permanece desabilitada e aparece na UI com a
explicação "Not verified on the current build profile".

Quando todas as capacidades pretendidas estiverem verificadas, mude
`status` para `ProfileStatus::Supported`.

### 7. Registrar evidência

Adicione uma linha ao histórico abaixo e abra o PR usando o template.

## Histórico de builds

| Perfil | SHA-256 | Status | Verificado em | Evidência |
| --- | --- | --- | --- | --- |
| `fsd-8e22e371` | `8e22e371…c77a41` | `supported` | 2026-08-14 | Suíte live-tests + verificação manual das mutações |

## Checklist de PR para uma build nova

- [ ] Hash confirmada com `Get-FileHash` e registrada em minúsculas
- [ ] Perfil novo criado como `Draft` com `Capabilities::NONE`
- [ ] Módulo registrado em `profiles/mod.rs` e em `ALL`
- [ ] Offsets e assinaturas conferidos **nesta** build, não herdados
- [ ] `cargo fmt`, `cargo clippy -D warnings` e `cargo test` verdes
- [ ] Live tests executados com save descartável
- [ ] Cada mutação permanente verificada manualmente no jogo
- [ ] Capacidades habilitadas **apenas** onde houve verificação
- [ ] `status` promovido só depois de tudo acima
- [ ] Linha adicionada ao histórico de builds
- [ ] Release notes indicam as builds suportadas

## Depreciar uma build

Mude `status` para `ProfileStatus::Deprecated`. O perfil continua no binário
para que o status bar consiga dizer "conheço esta build, mas não a suporto
mais" — bem mais útil ao usuário do que "build desconhecida".
