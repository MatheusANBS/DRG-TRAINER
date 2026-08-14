# DRG Credits

Aplicativo desktop local com backend Rust/Tauri e interface React/Vite/Tailwind.

Os prototipos Python (`drg_credits_monitor.py` e `drg_credits_gui.py`, quando presente)
continuam no diretorio como referencia da descoberta original.

## Desenvolvimento

```powershell
npm install
npm run tauri dev
```

## Build

```powershell
npm run tauri build
```

O backend valida o SHA-256 da build conhecida antes de acessar os offsets. Leitura e
escrita usam handles separados; o handle com permissao de escrita existe apenas durante
o comando de alteracao.
