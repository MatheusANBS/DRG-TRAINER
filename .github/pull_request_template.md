## Resumo

Descreva a mudança e o motivo.

## Validação

- [ ] `npm run lint`
- [ ] `npm test -- --run`
- [ ] `npm run build`
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --locked --all-targets -- -D warnings`
- [ ] `cargo test --locked` (sem o jogo aberto)
- [ ] Alterações de memória foram testadas primeiro em modo somente leitura
- [ ] Nenhum save, backup, token ou artefato de build foi incluído

## Compatibilidade de build

- [ ] Esta mudança **não** depende de uma build nova do jogo

Se depender, marque abaixo e siga [docs/build-support.md](../docs/build-support.md):

- [ ] Perfil criado como `Draft` com `Capabilities::NONE`
- [ ] Offsets e assinaturas conferidos **nesta** build, não herdados
- [ ] Live tests executados com save descartável
- [ ] Capacidades habilitadas apenas onde houve verificação
- [ ] Linha adicionada ao histórico de builds

## Arquitetura

- [ ] Nenhuma chamada Win32 de processo/memória fora de `infrastructure::process` (SPEC-001)
- [ ] Nenhum offset, hash ou assinatura fora de `build_profiles` (SPEC-002)
- [ ] Toda mutação permanente de save passa por `SaveTransaction` (SPEC-020)
- [ ] Nenhum componente chama `invoke` fora de `services/trainer-api.ts` (SPEC-012)
- [ ] Toda ação nova declara sua semântica em `types/actions.ts` (SPEC-015)

Mudou alguma decisão arquitetural durável? Adicione ou atualize um ADR em
[docs/adr/](../docs/adr/).

## Documentação

- [ ] Docs afetados foram atualizados (`docs/architecture.md`, `docs/testing.md`,
      `docs/build-support.md`, `docs/frontend-design.md`, `SECURITY.md`)
- [ ] Não se aplica
