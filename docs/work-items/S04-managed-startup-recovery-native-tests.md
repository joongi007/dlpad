# S04: Windows 관리 startup 복구 실패 인수

- 상태: in-progress
- 분류: safety
- 우선순위: P0
- 요구사항: R03, R05, R08, R13
- 담당/수정 범위: diagnostic_cache; 신규 tools/windows_managed_recovery_tests.rs 및 본 문서만
- 선행 작업: root recovery module와 RecoverySeed, 공개 ToolManager/begin_execution

## 사용자 결과

실제 원 관리 scope를 생성한 뒤 미완료·손상·foreign replacement 상태의 자동 삭제를 거절하는 Windows native 실패 회귀다.

## 계약과 범위

공개 ToolManager 생성과 begin_execution으로 원 v3/journal/receipts를 만든다. 내부 synthetic journal fixture를 복제하지 않는다. 원 소유 임시 파일에만 format/ACL/replacement fault를 넣는다. actual receipt 성공·실제 Job crash 회수는 별도 담당이다.

## 인수 조건

absent, live Busy, dropped pristine, legacy/unknown/oversize/digest tamper, foreign receipt/lock, 넓어진 ACL에서 recovered=true가 없어야 한다. 원 fixture 파일 snapshot과 별도 source/sentinel을 byte-for-byte 보존한다. live scope 종료 후 temp guard를 정리한다.

## 검증과 handoff

8개 실패 인수 테스트 작성 및 개별 rustfmt exit0. Windows native 실행은 root 담당이며 아직 실행하지 않았다. live journal의 원 native lock으로 read가 거절될 경우에도 파일 크기와 원 OS 오류를 snapshot에 포함하여 관측 누락을 숨기지 않는다. 정상 readable 파일은 byte-for-byte 대조한다. Cargo 및 다른 파일 변경 없음.

주 담당 실제 인수: `windows-managed-startup-recovery-workspace-first` Windows GNU CPU fmt/clippy/test/release PASS, 신규 보존 회귀 8개 실제 통과. `windows-paired-parent-crash-sentinel-retry`는 공개 Repository 실행 중 원 부모 HANDLE TerminateProcess73, 실제 두 도메인 receipt·원 관리/쿠키 복구·재사용·DB Interrupted·원 Job 자연 Active0·private desktop 종료/input desktop 불변을 통과했다. root-review.json에서 원 receipt 5개 SHA/256바이트, 원 cookie/tool/sentinel 보존, DB Interrupted/artifact0·인증 비밀 부재, 소유 journal/pending/owner 실제 부재 및 당시 source SHA를 독립 대조했다. 초기 fixture의 foreign sentinel rejected0 기대 오류는 기존 API 계약의 scanned2/recovered1/rejected1로 고쳤으며 두 원 실패를 보존했다. 관측기의 빠른 종료 뒤 미수신 pipe 출력을 먼저 읽도록 bounded drain을 보강했다. 실제 GUI 전체 crash·sleep/reboot와 전체 P0 인수는 별도다.
