# R04: 익명 공개 VOD 후보 metadata 관측 준비

- 상태: proposed / 준비 완료, 실제 실행 0
- 분류: validation
- 우선순위: P0
- 요구사항: R04, R05, R11
- 담당/수정 범위: p0_audit; 이 scratch namespace만. Root가 실제 Cargo/Windows 실행 통합
- 선행 작업: 현재 source32 whole 및 독립 Root review·기본 3PE·원 pinned yt-dlp 일치. Root가 CI/Cargo 작업과 순차 조정

## 사용자 결과

공개 후보12815830/1946995가 현재 익명 metadata로 조회되는지 판별할 작은 실행 후보. 원 사용자15441192의404는 그대로 보존한다.

## 계약과 범위

기존 process::run_metadata/Windows guardian Job와 기존 Pin 원HANDLE·SHA·NativeFileIdentity 검증을 재사용. 프로젝트 source/Cargo/lock 변경0. Scratch Cargo는 기존 package/dependencies/lock 그대로, 충돌 없는 별도 bin명만 추가. 새 dependency0. 기존 build target의 별도 debug exe만 생성하며 기본 release3PE를 쓰지 않는다.

## 인수 조건

원whole/source/3PE/도구SHA 불일치 즉시 거부. 익명·config/plugin/JS discovery/remote components 차단; cookie/auth환경·기존home·proxy 재사용0. argv 고정 두 후보, 각30초, retries0, stdout8MiB/로그16KiB. 원 stderr/JSON/CDN서명URL 저장0. title≤512UTF8bytes, id/duration/extractor 및 exit/분류만 정제 요약; 일부 후보 실패해도 이미 정제한 부분 행 보존. ownership 불명확이면 다음 프로세스 금지. 전체 영상/GUI/실제 다운로드 인수로 승격금지.

## 검증과 handoff

argv.json을 Root만 실행. 실제 현재 수행: Python AST2개 PASS; 원Cargo.lock byte동일 static 확인. Cargo/Rustfmt/native/network 실행0, 실제 Rust compile 및 Windows version/metadata 미검증. 원 도구2026.08.19 provenance SHA를 실행시검증하며 실제 --version도 기록한다. 도구 업데이트/원PE/원증거 삭제0.

Root 실행 예정: 현재v32 source/3PE/whole Root검토와원pinned Windowsyt-dlp에바인딩한다. 실제영상다운로드/인증/쿠키/원metadata저장0. 기존사용자15441192 HTTP404관측을지우거나원인확정하지않고,공식공개후보2개의현재metadata만별도로관측한다. 사용자Documents나기존미디어는사용하지않는다. 독립CI는별도target에서진행중이며이fixture는Root가공유MSVCtarget변경을조정한다.

Root 실제3행 수집·2operations0 후33pins·675source·원logs를 독립 대조했다.12815830은 chzzk:video/93초/exit0,1946995는 exit1/HTTP404 관측이며 원인 미확정이다. 인증·raw JSON/stderr 저장·미디어 다운로드·GUI 시험0이다. [Root 검토](../validation/evidence/r04-public-vod-metadata-candidates-v1-20261016/root-review.json).
