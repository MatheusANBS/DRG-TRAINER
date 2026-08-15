# Security policy

## Versões suportadas

| Versão | Suporte |
| --- | --- |
| 0.1.x | Sim |
| Anteriores | Não |

## Relatando um problema

Use um GitHub Security Advisory privado no repositório para relatar vulnerabilidades. Evite abrir uma issue pública antes da análise, especialmente quando o relatório envolver escrita arbitrária em memória, caminhos de save ou execução remota no processo do jogo.

Inclua, quando aplicável:

- versão do trainer;
- SHA-256 do `FSD-Win64-Shipping.exe`;
- versão do Windows;
- passos mínimos para reprodução;
- impacto observado;
- logs sanitizados, sem saves ou informações pessoais.

Não envie arquivos `.sav`, tokens, credenciais ou dumps completos de memória.

## Escopo

São relevantes problemas no aplicativo, no processo de build, nas permissões do GitHub Actions e no tratamento dos backups. Alterações de comportamento causadas exclusivamente por uma nova build do jogo devem ser tratadas como incompatibilidade, não como vulnerabilidade.

## Superfície de ataque e controles

### Fronteira Win32 (SPEC-001)

Todo acesso a processo e memória está confinado a `src-tauri/src/infrastructure/process/`. Nenhum módulo de domínio chama `ReadProcessMemory`, `WriteProcessMemory`, `VirtualAllocEx` ou `CreateRemoteThread` diretamente. Os invariantes são:

- handles fechados por RAII, sem caminho manual de liberação;
- processo aberto somente para leitura por padrão; escrita e execução remota exigem `Access::ReadWrite` explícito;
- todo intervalo é validado antes do dereference (endereço mínimo de usuário, teto de transferência, ausência de overflow);
- leitura ou escrita parcial é erro, nunca sucesso silencioso;
- nenhuma rotina nativa é executada sem validar o prefixo de código declarado no perfil da build.

### Perfis de build (SPEC-002)

Offsets, assinaturas e endereços nativos vivem apenas em `src-tauri/src/build_profiles/`. O runtime resolve o perfil pelo SHA-256 do executável antes de expor qualquer recurso dependente de memória. Build desconhecida, ou perfil ainda não promovido a `supported`, mantém todas as capacidades desabilitadas.

### Mutação de save (SPEC-020)

Toda alteração permanente passa por `SaveTransaction`: backup versionado antes da escrita, verificação após, e caminho de restauração incluído em qualquer mensagem de erro pós-backup. Backups nunca são sobrescritos nem removidos automaticamente.

### Content Security Policy (SPEC-010)

A WebView roda com CSP explícita em `src-tauri/tauri.conf.json`:

| Diretiva | Valor | Motivo |
| --- | --- | --- |
| `default-src` | `'self'` | Nenhuma origem remota é permitida por padrão. |
| `script-src` | `'self'` | O bundle é local; não há script inline em produção. |
| `style-src` | `'self' 'unsafe-inline'` | Radix e Tailwind injetam estilos inline em runtime. |
| `img-src` | `'self' asset: data:` | Ícones locais e SVG embutidos. |
| `font-src` | `'self' data:` | Fonte Geist empacotada; Vite pode inlinar arquivos pequenos. |
| `connect-src` | `'self' ipc: http://ipc.localhost` | Somente o IPC do Tauri. Sem rede. |
| `object-src`, `frame-src`, `worker-src` | `'none'` | Recursos não utilizados pelo trainer. |
| `form-action` | `'none'` | O app não envia formulários. |
| `frame-ancestors` | `'none'` | A janela não pode ser embutida. |

`devCsp` acrescenta apenas o servidor do Vite (`localhost:1420`) e `'unsafe-inline'` para o HMR.

### Capabilities do Tauri (SPEC-010)

`src-tauri/capabilities/default.json` declara o conjunto mínimo:

| Permissão | Motivo |
| --- | --- |
| `core:app:default` | Metadados do app usados pela WebView na inicialização. |
| `core:event:default` | Sistema de eventos usado pelo mecanismo de callback do `invoke`. |
| `core:window:default` | A janela única é criada pela configuração e consultada pelo frontend. |
| `core:webview:default` | Ciclo de vida da própria WebView. |

Não são concedidas permissões de filesystem, shell, opener, HTTP ou clipboard. Os comandos do trainer são comandos próprios registrados em `generate_handler!`, sem exposição adicional. `withGlobalTauri` está desabilitado: não existe `window.__TAURI__` para código de página.

Qualquer inclusão nesta lista deve vir acompanhada de justificativa no PR.

## Cadeia de dependências (SPEC-009)

### Controles

| Controle | Onde |
| --- | --- |
| `npm audit --audit-level=high` | Gate `dependency-audit` no CI |
| `cargo audit --deny warnings` | Gate `dependency-audit` no CI |
| Lockfiles obrigatórios (`npm ci`, `--locked`) | Todos os jobs |
| GitHub Actions pinadas por commit SHA | Todos os workflows |
| `permissions` mínimas por workflow | `contents: read`; só o job de publicação eleva para `contents: write` |
| Dependabot | `.github/dependabot.yml` |
| CodeQL | Verificação padrão do repositório |

### Política de exceção de advisory

**Vulnerabilidades nunca são ignoradas.** Só advisories informativas
(`unmaintained`, `unsound`, `notice`) podem virar exceção, e apenas quando não
alcançam o binário distribuído.

Toda exceção vive em `src-tauri/.cargo/audit.toml` e precisa declarar:

1. o motivo do advisory;
2. por que ele não afeta este projeto;
3. o gatilho de revisão e a próxima data de revisão.

As exceções atuais são todas transitivas do Tauri: as bindings GTK3 (que só são
compiladas em Linux — o trainer é Windows-only e usa WebView2) e utilitários de
macro sem manutenção. Nenhuma delas é uma vulnerabilidade. Revisão a cada
atualização de major/minor do Tauri e, no mínimo, a cada seis meses.

Uma exceção sem prazo é uma exceção permanente disfarçada — o arquivo é
rejeitado em review se faltar a data.
