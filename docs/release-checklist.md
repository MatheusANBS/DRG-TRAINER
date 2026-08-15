# Checklist de release

Preenchido a cada versão (SPEC-022). Critérios **bloqueantes** impedem a
publicação; **recomendados** são registrados, não travam.

Versão: `_______`  ·  Commit: `_______`  ·  Data: `_______`

## Bloqueantes — automatizados

Verificados pelo job `gates` de `.github/workflows/release.yml`.

- [ ] `npm run lint` sem erros
- [ ] `npm test -- --run` verde
- [ ] `npm run build` sem erros
- [ ] `cargo fmt --all -- --check` limpo
- [ ] `cargo clippy --locked --all-targets -- -D warnings` sem warnings
- [ ] `cargo test --locked` verde **em máquina sem o DRG instalado**
- [ ] `npm audit --audit-level=high` sem achados
- [ ] `cargo audit --deny warnings` sem achados fora das exceções documentadas

## Bloqueantes — build profile

- [ ] Todo perfil embarcado está com `status: supported`
- [ ] Nenhum perfil `draft` ou `verified` no binário público
- [ ] Cada perfil tem evidência registrada em [build-support.md](build-support.md)
- [ ] Build desconhecida bloqueia ações e é reportada no status bar
- [ ] Release notes indicam explicitamente as builds suportadas

**Falha em verificação de build profile bloqueia a release.**

## Bloqueantes — matriz de regressão

Executada manualmente, com **save descartável**, na build suportada.

### Conexão

- [ ] Jogo fechado: status mostra `GAME NOT RUNNING` e o executável esperado
- [ ] Jogo aberto na build suportada: `GAME ATTACHED` + `BUILD VERIFIED` + perfil
- [ ] Build não suportada: `BUILD UNSUPPORTED` + aviso, todas as ações desabilitadas
- [ ] Fechar o jogo com o trainer aberto volta para `GAME NOT RUNNING` sem erro
- [ ] Reabrir o jogo reanexa sem reiniciar o trainer

### Inventory

- [ ] Créditos são lidos e acompanham mudanças no jogo
- [ ] Escrita de créditos é verificada e refletida no jogo
- [ ] Os 14 recursos aparecem com valores corretos
- [ ] Adicionar a um recurso funciona e cria backup
- [ ] Adicionar a todos os recursos funciona e cria um único backup

### Player

- [ ] Max Class Level leva as quatro classes ao nível 25
- [ ] Promoções existentes são preservadas pelo Max Class Level
- [ ] Promote All Classes promove as quatro e zera o XP
- [ ] Unlock All Perks conclui e persiste

### Weapons

- [ ] Infinite Magazine acompanha a troca de arma
- [ ] Infinite Magazine desliga e devolve o comportamento normal
- [ ] Weapon Damage acompanha a arma e os componentes ativos
- [ ] Weapon Damage desliga e restaura o dano original
- [ ] Hotkeys globais alternam os dois toggles
- [ ] Hotkey inválida é recusada sem perder a anterior
- [ ] Unlock All Weapons conclui e persiste
- [ ] Unlock All Gear Modifications conclui e persiste
- [ ] Unlock All Overclocks & Cosmetics conclui e persiste

### Save e backup

- [ ] Toda mutação permanente cria um diretório de backup novo
- [ ] Dois backups na mesma operação não se sobrescrevem
- [ ] Restaurar um backup devolve o save ao estado anterior
- [ ] Falha pós-backup informa o caminho de restauração

### Interface

- [ ] Sem overflow horizontal em 760x590
- [ ] Sem overflow horizontal em 820x620 e 940x660
- [ ] Textos não colidem com scaling do Windows em 100%, 125% e 150%
- [ ] Toda ação desabilitada explica o motivo
- [ ] Ações permanentes exibem `PERMANENT` antes do clique
- [ ] Confirmação aparece antes de toda mutação permanente
- [ ] Foco é visível em toda navegação por teclado

### Instalação

- [ ] O executável roda em máquina limpa com WebView2 instalado
- [ ] Primeira execução sem erros no console
- [ ] Aviso do SmartScreen documentado (enquanto não houver assinatura)

## Bloqueantes — distribuição

- [ ] `SHA256SUMS.txt` gerado pelo CI (não localmente)
- [ ] `build-manifest.json` registra commit, tag e perfis suportados
- [ ] Checksum publicado confere com o binário publicado
- [ ] Nenhuma chave, certificado ou save no repositório ou nos artefatos
- [ ] `CHANGELOG.md` atualizado

## Recomendados

- [ ] Screenshots do README refletem a versão
- [ ] Live tests executados na build suportada
- [ ] Scorecard de qualidade atualizado ([quality-scorecard.md](quality-scorecard.md))
- [ ] ADRs criados para decisões arquiteturais desta versão

## Assinatura

Nome: `_______`  ·  Data: `_______`

Declaro que os critérios bloqueantes acima foram verificados neste commit.
