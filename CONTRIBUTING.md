# Contribuindo

## Antes de começar

Este projeto lê e escreve na memória de outro processo e altera saves. Um bug
aqui custa o progresso de quem usa. Duas leituras curtas antes do primeiro PR:

- [docs/architecture.md](docs/architecture.md) — onde as coisas ficam e por quê
- [docs/testing.md](docs/testing.md) — como validar sem o jogo aberto

## Ambiente

- Windows 10 ou 11 x64
- Node.js 20+
- Rust stable com target MSVC
- Microsoft Edge WebView2 Runtime

```powershell
npm install
npm run tauri dev
```

Para trabalho puramente visual, `npm run dev` sobe o frontend no browser com um
backend simulado — sem o jogo e sem Tauri.

## Os gates

Rode antes de abrir o PR. São exatamente os mesmos do CI.

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

`cargo test` precisa passar **com o jogo fechado**. Se um teste seu exige o DRG
aberto, ele pertence ao nível `live-tests`.

## Regras que não se negociam

Elas existem porque cada uma já foi, ou seria, a causa de um bug caro.

1. **Win32 só em `infrastructure::process`.** Domínio conversa com traits.
   ([SPEC-001](docs/adr/001-win32-boundary.md))
2. **Offsets, hashes e assinaturas só em `build_profiles`.** Se você digitou um
   número hexadecimal fora de um perfil, provavelmente está no lugar errado.
   ([SPEC-002](docs/adr/002-build-profiles.md))
3. **Nenhuma escrita reportada como sucesso sem releitura.**
4. **Nenhuma rotina nativa chamada sem validar a assinatura.**
5. **Nenhuma mutação permanente sem `SaveTransaction`.**
   ([SPEC-020](docs/adr/003-save-transactions.md))
6. **Nenhum componente chama `invoke`.** Só `services/trainer-api.ts`.
7. **Nenhuma tela calcula `disabled` sozinha.** Use `blockedReason`.
8. **Toda ação declara semântica** (`runtime`, `standard`, `persistent`).

## Trabalhando em uma build nova do jogo

Siga [docs/build-support.md](docs/build-support.md) passo a passo. O passo mais
importante, e o mais fácil de pular: ao copiar um perfil, **zere as
capacidades**. Herdar offsets é o erro que o processo inteiro existe para
impedir.

## Testes

Cubra o caminho de erro, não só o feliz — metade das garantias deste projeto é
sobre **recusar** operações.

No frontend, consulte por papel e texto, nunca por classe CSS. Assim o
polimento visual não quebra a suíte.

## Estilo

- Rust: `rustfmt` padrão, Clippy sem warnings.
- TypeScript: ESLint, sem `any`, sem promise flutuante.
- Comentários explicam **por quê**, não o quê. Se o código precisa de um
  comentário para dizer o que faz, geralmente falta um nome melhor.
- Mensagens de erro do backend em português (é onde o diagnóstico técnico mora);
  textos de interface em inglês. `src/lib/errors.ts` faz a ponte.

## Segurança

Vulnerabilidades vão por GitHub Security Advisory privado, não por issue. Ver
[SECURITY.md](SECURITY.md).

Nunca inclua em um PR: arquivos `.sav`, backups, dumps de memória, tokens ou
certificados.
