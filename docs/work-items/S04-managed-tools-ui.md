# S04: 관리 yt-dlp 설치·버전·제거 UI

- 상태: in-progress
- 분류: feature
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: 주 담당; app/managed.rs, app/main.rs·catalog.rs, 전용 문서; GUI harness는 diagnostic_cache 담당 신규 script
- 선행 작업: S04-direct-download, S04-managed-selection (backend 담당 조정)

## 사용자 결과

공식 pin의 크기·license·검증 상태를 확인하고 설치한다. 설치된 관리 버전을 선택·제거하고 검증한 active 경로를 다운로드 입력에 명시적으로 적용한다. 외부 도구는 관리 목록이나 제거 대상으로 전환하지 않는다.

## 계약과 범위

ADR-0003. Linux x86_64/Windows x64/macOS 공식 직접 실행 파일만 고정한다. 아카이브/FFmpeg/JS runtime은 후속. TLS 다운로드와 DB/hash/version I/O는 worker, UI와 설치 작업의 busy/cancel 독립. 설치 중 다운로드 시작/재개와 변경·제거를 막는다. active 제거는 backend와 UI에서 거절. source/license pin을 미검증 OS 실행 완료로 표현하지 않는다.

## 인수 조건

크기와 선택 비용 안내, HTTPS+SHA256+version 이후에만 active, 실패/취소/재선택 기존 도구 보존, 명시적 사용 경로 적용. GUI는 가상 화면에서만 검증한다. 정상 Git 없이 기존 .git 보존.

## 검증과 handoff

구현 중. backend API는 diagnostic_cache가 담당하며 같은 파일을 수정하지 않는다.

### 중간 통합 증거

관리 화면 compile/check와 당시 workspace tests/release를 실행했다. 실제 Xvfb [첫 화면](../validation/iced-managed-tools.png)/[측정](../validation/iced-managed-tools.json)에서 크기·license·고정 버전·빈 목록을 확인했다. 앱9,667,936 bytes(기존7,974,752보다1,693,184 증가)는 UI가 HTTP/TLS backend를 호출한 뒤의 통합 비용이다. RSS69,420KiB/3초CPU0%는 cold idle 짧은 표본이며 다운로드 peak가 아니다.

다운로드 중 설치/삭제/선택 차단, 설치 중 새 다운로드 gate, 실패 목록 보존, 종료 ack 대기, 적용 전 active 재검증 worker를 구현했다. Windows 네이티브 clippy가 큰 Managed 메시지를 발견해 Box로 한정하고 모든 Task/Element map을 공통 생성 함수로 바꿨다. 원인별 정제된 한국어 실패와 로컬 ToolDiscovery 보고서를 연결했다. UI의 실제 설치·선택·제거/실패 흐름 검증이 아직 남아 implemented로 바꾸지 않는다. Vendor IME 실험은 현재 진행 중이며 임시 trace/불필요한 patch를 최종 빌드 전에 정리한다.

### 설치 회복 UI 연결

목록 갱신/관리 작업 worker에서 recover_staging을 먼저 호출하며 종료·파일 증거가 확인된 잔여물만 정리한다. 소유 증거가 없거나 실행 중인 잔여물은 삭제하지 않고 보존 개수를 중앙 문자열로 안내한다. 실패한 회복은 기존 UI 목록을 유지한다. 별도 backend schema 변경은 없다. 전체 workspace 시험에서 직접 다운로드 실패 경로 2개가 실패해 담당자가 재현 중이며 성공으로 기록하지 않는다.
