# S04: 관리 패키지 선택의 취소와 기한 경계

- 상태: in-progress
- 분류: fix
- 우선순위: P0
- 요구사항: R05, R08
- 담당/수정 범위: root — tools/package_inventory.rs, tools/deno_version.rs, tools/ffmpeg_version.rs; diagnostic_cache — /tmp 테스트 초안만
- 선행 작업: 현재 전체 workspace collector terminal 및 source freeze 해제

## 사용자 결과

Deno/FFmpeg 선택에서 큰 component SHA 검사 중 취소와 총 기한을 확인한다. DB 쓰기 대기 뒤 취소가 확인되면 active 변경을 rollback하여 기존 선택을 유지한다.

## 계약과 범위

기존 metadata inventory와 다른 caller를 유지하고 verify_package_with_checkpoint callback을 내부 추가한다. 초기 경계·directory entries·components·각64KiB read 직전·성공 반환 전을 검사한다. 두 runtime selector는 동일 deadline/cancel을 전달하고 transaction execute 후 commit 직전 확인한다. blocking filesystem/SQLite 호출 중 즉시 중단 또는 commit과 취소의 원자적 선후 보장은 주장하지 않는다. 공급 trust, 실행 pin lease, OS별 실행 및 UI는 별도 후속이다.

## 인수 조건

large component hash 중 Cancelled/TimedOut 반환 후 추가 read/checkpoint가 없고 원 파일·metadata·foreign sentinel을 보존한다. 작은 fixture 대비64KiB마다 callback 증가를 검증한다. SQLite write 대기 중 취소 fixture는 기존 active를 보존한다. parser/실제 runtime 성공·실패 보호 회귀 유지 및 strict Clippy 통과.

## 검증과 handoff

현재 source freeze 중 /tmp/dlpad-selection-checkpoint-patch.py와 /tmp/dlpad-package-checkpoint-tests.rs로 구현/회귀 초안을 준비했다. 프로젝트 Rust 적용 및 실제 검증은 아직 수행하지 않았다.

### 구현 및 실제 scoped 검증

Linux 전체 collector 기본/CPU9operation terminal0와 source378개 독립대조 불일치0 확인 후 source freeze를 해제했다. package_inventory에 checkpoint callback을 추가하고 Deno/FFmpeg의 전후 SHA 검사와 commit 직전 경계에 연결했다. 각64KiB read 전 검사이며 blocking read 자체를 강제 중단하지 않는다. tools library76PASS/1ignored, actualSQLite busy callback을 관측한 뒤 cancel→unlock→rollback/prioractive 보존 회귀 PASS, large component Cancelled/TimedOut 8조합 및 read 경계 회귀 PASS. engine all-target Clippy exit0. 현재 변경 이후 전체 workspace/Windows 재검증은 별도이며 앞 snapshot 결과를 확대하지 않는다.
