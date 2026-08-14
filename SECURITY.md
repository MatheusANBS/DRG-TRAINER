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
