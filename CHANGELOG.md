# Changelog

## Não lançado

Reestruturação para o marco de qualidade 9+, sem mudança de comportamento das
funcionalidades existentes.

### Segurança e compatibilidade

- Acesso Win32 a processo e memória confinado a uma única camada, com validação
  de intervalo antes de todo dereference e escrita sempre verificada por
  releitura.
- Perfis de build (`BuildProfile`) passam a concentrar hash, offsets,
  assinaturas e catálogos. A build é resolvida por SHA-256 antes de qualquer
  operação dependente de offset.
- Capacidades verificadas por build: uma ação não validada no perfil ativo
  permanece desabilitada, com o motivo explicado na interface.
- Mutações permanentes passam por uma transação de save única, com backup
  versionado que nunca sobrescreve outro.
- CSP ativa e capabilities do Tauri reduzidas ao mínimo necessário.

### Qualidade

- Backend reorganizado em `app`, `domain`, `infrastructure`, `build_profiles` e
  `shared`.
- Arquitetura de testes em três níveis; `cargo test` roda sem o jogo instalado.
  Testes que exigem o DRG ficam atrás da feature `live-tests`.
- Frontend com ESLint e Vitest; `App.tsx` reduzido a composition root, com
  `invoke` restrito a uma única camada de serviço.
- CI dividido em gates independentes, com Clippy como erro, testes offline e
  auditoria de dependências Rust.
- Release com checksums e manifesto de build gerados pelo CI, e estágio de
  assinatura Authenticode pronto para ser ligado.

### Interface

- Barra de status operacional: processo, build e perfil legíveis em texto.
- Semântica explícita de ação (`runtime`, `standard`, `persistent`) com badge
  `PERMANENT` antes do clique.
- Tokens de tipografia e espaçamento; nenhum texto funcional abaixo de 9px.
- Breakpoint responsivo alinhado ao mínimo real da janela (820px).
- Estados assíncronos padronizados, sem disparo duplicado, com erros mapeados a
  partir de códigos estáveis do backend.

### Documentação

- `docs/architecture.md`, `build-support.md`, `testing.md`, `release.md`,
  `frontend-design.md`, `save-restore.md`, `release-checklist.md` e
  `quality-scorecard.md`.
- ADRs das decisões centrais em `docs/adr/`.
- `CONTRIBUTING.md`, template de PR e templates de issue.

## 0.1.0 — 2026-08-14

Primeira versão pública.

### Incluído

- Monitoramento e escrita verificada de créditos.
- Adição individual ou em lote dos 14 recursos persistentes.
- Infinite Magazine com acompanhamento da arma equipada.
- Weapon Damage com acompanhamento dos componentes ativos.
- Hotkeys globais configuráveis para recursos em tempo real.
- Unlock de armas, perks, modificações, overclocks e cosméticos.
- Nível máximo e promoção das quatro classes.
- Backup automático antes de mudanças permanentes no save.
- Validação da build do jogo por SHA-256.
- Resolução dinâmica da instância ativa de `FSDSaveGame`.
