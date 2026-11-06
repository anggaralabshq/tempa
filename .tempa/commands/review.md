---
description: Review perubahan di branch ini terhadap kontrak tempa
---
Review diff branch ini terhadap `main` ($ARGUMENTS bila diberikan).

1. Baca AGENTS.md (Konvensi, Area terlarang) dan plan aktif di docs/plans/.
2. Untuk setiap perubahan perilaku, pastikan ada test yang akan gagal tanpa perubahan itu.
3. Cari risiko kehilangan data: penulisan file, penghapusan, penimpaan file milik pengguna.
4. Laporkan temuan sebagai `path:line — masalah — saran`, urut dari paling berat.
