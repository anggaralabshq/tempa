# Tempa — setup untuk DevOps (sekali saja)

## 1. Step Bitbucket Pipelines

Tambahkan ke `bitbucket-pipelines.yml`:

```yaml
pipelines:
  pull-requests:
    '**':
      - step:
          name: tempa contract
          image: <registry>/tempa:{version}
          clone:
            depth: full
          script:
            - tempa check --contract-only
```

Step ini hanya memeriksa drift dan kontrak dokumen, tanpa secret dan tanpa menjalankan test. Lint/test tetap di step yang sudah ada.

`<registry>` diisi setelah lokasi hosting image diputuskan. Versi image harus sama dengan `tempa_version` di `tempa.toml`.

## 2. Merge check (Repository settings → Branch restrictions → branch tujuan)

- Minimum successful builds: 1
- Minimum approvals: 1
- Centang "Prevent a merge with unresolved merge checks" (Premium)

## 3. Jangan

- Jangan menerima build status dari mesin developer sebagai pengganti step ini.
