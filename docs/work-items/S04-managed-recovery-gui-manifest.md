# S04: Windows 복구 GUI 인수 manifest

- 상태: in-progress
- 분류: validation
- 우선순위: P0
- 요구사항: R05, R08, R11
- 담당/수정 범위: root; scripts/windows_managed_recovery_gui_manifest.py, scripts/windows_paired_parent_crash_observer.cs 및 본 문서
- 선행 작업: 합성 완료/보류/손상 fixture와 private desktop GUI observer

## 사용자 결과

실제 앱 시작 전후 원본 보존과 소유 복구 기록 부재를 검증할 manifest를 생성한다.

## 계약과 범위

프로젝트 evidence 아래 owned-fixture와 원 capability를 확인하고 원 managed namespace/cookie owner만 추적한다. create-only JSON을 작성하며 실제 사용자 데이터·계정·영상은 읽지 않는다. GUI observer는 독립 파일 SHA/상태/비공개 화면을 검사하고 DB 상태는 root가 별도로 확인한다.

## 인수 조건

완료 사례는 원본 보존과 관리10개 이름/cookie owner 부재를 요구한다. 보류·손상 사례는 원 기록 byte-for-byte 보존을 요구한다. 잘못된 namespace/symlink/기존 manifest는 거절한다.

## 검증과 handoff

구현 중. 실제 GUI 실행은 fixture/observer handoff 후 source freeze에서 수행한다.

주 담당 실제 Windows 인수: 첫 준비 수집에서 fixture·두 C# observer 컴파일 및 원 부모 종료 후 receipt 보존 PASS. 실제 GUI pending/malformed의 재확인·원 보호 SHA·malformed 시작 차단·원 Job 자연 종료/private desktop 종료/input desktop 유지 PASS, pending-malformed-root-review.json으로 독립 대조했다. completed 첫 수집은 .NET4 경로 길이 제한으로 실패하여 원 실패를 보존했다. 짧은 fresh `mg-gui2` 합성 경로에서 새 부모 종료 기록으로 completed GUI 회수/실제 비정상 중단 이력·원 보호 SHA·소유 기록 부재·DB Interrupted를 통과했고 root-review.json으로 독립 대조했다. 당시 고정 앱 SHA709de3이며 이후 pending 안내를 항상 보이는 status에 보강했다. 최신 Windows 전체 fmt/clippy/test/release PASS, 최신 pending 화면 재검증은 진행 중이다. 실제사이트/IME/절전/전체 P0 완료로 확대하지 않는다.

최신 pending-visible 실제 GUI도 PASS. 주 담당 startup.png를 직접 읽어 첫 viewport의 status에 중앙 관리 보류 안내가 보이는 것을 확인했다. 재확인·원 파일 SHA·자연 Job 종료 및 최신 고정 binary/current source SHA도 독립 대조했다. 최신 Windows 전체 네 검증과 문서 612개 검사 PASS, 전체 P0는 미완료다.
