# B-R5 S03 Release artifact — IMPLEMENTED_UNVERIFIED 2026-09-25

Slice: `.deb` do batch pivot com checksum e metadados. Instalação em sistema
limpo fica como probe manual (exige root; sem confirmação não se instala).

## Artefato (MEASURED, host i7-13620H, Pop!_OS 24.04)

* `src-tauri/target/release/bundle/deb/BrainRoot_0.0.14_amd64.deb`
* bytes=3533246
* sha256=`2f60cdd5c123de142d41f8139298bf7350137438cca793d418947f4fd8c9caaf`
* Depends: `libwebkit2gtk-4.1-0, libgtk-3-0`; Installed-Size 8078 KB
* Conteúdo: `usr/bin/brainroot` + `.desktop` + ícones (sem LICENSE/NOTICE ainda —
  gap registrado: empacotador precisa incluir, ver ADR 0010)

## Baselines do batch (mesmo host)

* App: READY-ok, fecha limpo, 0 órfãos; RSS 460–479MB / PSS 213–237MB
  (WebKit domina; budget ≤150MB segue FAIL conhecido desde B05).
* Sidecar `--pure`: 311–482MB por run (transiente, nunca residente; governor 60s).
* Frontend: JS 101KB (gzip 34KB); binário 8.2MB.
* Soak sidecar 5/5; portão `check-fast.sh` verde (169 Rust + 51 frontend).

## Gaps honestos antes de qualquer release pública

1. Instalação/remoção em ambiente limpo não executada (requer root).
2. LICENSE/NOTICE fora do `.deb`.
3. Preview/browser fora do registro do governor (S03 futuro).
4. Preços do catálogo sem timestamp visível na UI (só `stale`).

## Rollback

Apagar o `.deb` de `target/` (ignorado pelo git). Nenhum arquivo rastreado muda.
