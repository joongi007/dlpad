# 관리 보관 버전·기존 active 보존 GUI 검증

2026-10-01. [작업 계약](../work-items/S04-managed-retained-gui.md), [harness](../../scripts/managed_retained_probe.py).

Linux release binary SHA256 `7e3afa82764e4bc52b254f337db39c81431efede22c7dce9b3bb2aa8aafdfa3d`에서 Xvfb960x640/100%, 명시 기존 Malgun.ttf로 실행했다. 주 담당 binary를 사용하고 빌드·production 앱/engine/Cargo·기존 probe는 수정하지 않았다.

초기 root/manifest/active DB는 **synthetic-v1/v2 executable의 metadata/hash GUI fixture**다. 소유 marker와 각 digest/정확 path를 seed하고 synthetic-v1을 active로 기록했다. 이것은 공식 version 설치 또는 `--version` 실행 검증 증거가 아니다. 임시 data/media/external 파일은 sibling이며 실제 영상·계정·외부 설치·사용자 파일을 사용하지 않았다. 원본 공식 서버/기존 설치를 변경하지 않았다.

```sh
DLPAD_TEST_FONT=<fixture-font> python3 scripts/managed_retained_probe.py target/release/dlpad /tmp/new-retained-select.json --mode select-remove --timeout 30
DLPAD_TEST_FONT=<fixture-font> python3 scripts/managed_retained_probe.py target/release/dlpad /tmp/new-retained-cancel.json --mode cancel
DLPAD_TEST_FONT=<fixture-font> python3 scripts/managed_retained_probe.py target/release/dlpad /tmp/new-retained-hash.json --mode hash-failure
```

실제 screenshot로 inactive synthetic-v2 select740,380과 active 전환 후 old-v1 remove885,348을 확인했다. install216,272/cancel350,272도 해당 font/viewport의 실제 좌표다. 다른 환경에서는 `--mode inspect`와 명시 point 인자를 사용한다. visible desktop 옵션/fallback은 없다.

| 실제 UI 요청 | 자동 관측/직접 이미지 검토 | 증거 |
|---|---|---|
| 보관 v2 활성화→old v1 제거 | SQLite active v1→v2, old target directory 삭제, v2 bytes 및 external fixture 보존 |5.518초·[JSON](managed-retained-gui/select-remove.json)·[이전](managed-retained-gui/before.png)·[선택](managed-retained-gui/selected.png)·[제거](managed-retained-gui/removed.png)|
| 기존 active에서 새 설치 취소 | 공식 고정 HTTPS install staging 관측→중단, staging 없어짐, active-v1 DB/path/bytes·inactive-v2·external bytes 모두 불변 |4.567초·[JSON](managed-retained-gui/cancel.json)·[중단](managed-retained-gui/cancel.png)|
| 기존 active에서 같은 크기 SHA 불일치 | 이번 소유 임시 staging만 변조, 공식40,446,224 byte 완성 관측/digest 불일치, UI 검증 거절→staging 정리/기존 모든 bytes·active 보존 |10.544초·[JSON](managed-retained-gui/hash.json)·[거절](managed-retained-gui/hash.png)|

hash failure는 주 담당과 먼저 합의했다. 앱의 공식 HTTPS transfer가1MiB 이상 진행한 뒤 **소유 staging ELF e_ident padding offset9의 byte0→1만** 수정했다. 길이/inode 유지, writable handle 즉시 닫기, 실제 원본/외부 executable 변경 없음. 이후 같은 inode의 완성40,446,224 byte를 읽기 streaming SHA로 관측했다. digest `d6c62cbfd40e30e32f4b66e774c2b3e2541aa5a076875086dbc6f61a8f4a9944`는 공식 pin `58162f9bfdc27458ea47bfcb311cf47028f17d8154a8bf7d689861d46399230a`와 다르다. 화면에 “공급 파일의 크기 또는 SHA256이 일치하지 않습니다. 활성화하지 않습니다.”가 표시됐다.

UI가 HashMismatch/UnexpectedSize/DownloadLimit를 같은 문구로 묶으므로 정확 Rust error discriminant를 직접 계측했다고 주장하지 않는다. 동일 공식 size/identity와 다른 SHA, UI 검증 거절·기존 active 보존이 실제 관측 범위다. OCR은 없어 hash JSON의 native_gui_verified=false/error_text_requires_screenshot_review=true를 유지하며 문구는 이미지 직접 검토로 확인했다. 초기 hash 시험도9.606초에 같은 UI 거절/보존이 성공했지만 최종 size/digest를 관측하지 않아 [초기 JSON](managed-retained-gui/hash-initial.json)/[초기 화면](managed-retained-gui/hash-initial.png)을 별도 보존했다.

AST와 인자9건(정상 modes/timeout NaN·초과/잘못된 mode/화면 밖 좌표/desktop 거절) 통과했다. 모든 실제 GUI 프로세스와 Xvfb는 소유 범위에서 회수했고 temp fixture는 정리했다. 다른 OS, 실제 두 공식 배포 버전 rollback, 기존 공식 active에서 update 취소, 실제 영상 실행은 미검증이다. Synthetic metadata/hash UI 증거를 공식 실행 검증으로 바꾸지 않는다. 다음 binary에 이 SHA의 결과를 자동 적용하지 않는다. 전체 P0/3OS 출시 완료를 의미하지 않으며 주 담당 독립 검토가 남는다.
